use std::path::PathBuf;
use std::process::Command;

const COMMANDS: &[&str] = &[
    "set_items",
    "get_items",
    "set_register_widget",
    "reload_all_timelines",
    "reload_timelines",
    "request_widget",
    "create_widget_window",
    "close_widget_window",
    "set_widget_config",
    "set_widget_config_from_path",
    "get_widget_config",
    "widget_action",
    "poll_pending_actions",
    "report_receipt",
    "get_widget_diagnostics",
    "get_widget_trace",
    "flush_widget_trace",
    "set_widget_data",
    "set_widget_instances",
    "get_active_widgets",
    "start_activity",
    "update_activity",
    "end_activity",
    "list_activities",
    "activity_island",
    "get_activity_push_to_start_token",
    "set_widget_state",
    "get_widget_state",
    "register_activity_template",
    "get_widget_push_token",
    "can_promote_activities",
    "open_activity_promotion_settings",
];

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();

    let result = tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .try_build();

    // docs.rs sets DOCS_RS=1; prefer env over `--cfg docsrs` (rustc-args
    // would infect dependency crates and break docs builds — see Cargo.toml).
    let docs_rs = std::env::var_os("DOCS_RS").is_some();
    if !(docs_rs && target.contains("android")) {
        result.unwrap();
    }

    // Always compile the WidgetKit reload bridge on macOS host builds.
    // `swiftc -parse-as-library` already avoids a duplicate `main`; gating on
    // `codegen` left `macos_transport` with undefined symbols under `--all-features`.
    if target.contains("apple-darwin") {
        let sdk = macos_sdk_path();
        let arch = if target.contains("aarch64") {
            "arm64"
        } else {
            "x86_64"
        };
        compile_reload_bridge(&sdk, arch);
    }

    // Windows MSVC: tauri / tao import comctl32 v6 symbols (TaskDialogIndirect), which only
    // resolve with a Common-Controls 6 manifest; without it the integration test exes die at
    // load with STATUS_ENTRYPOINT_NOT_FOUND. Apps get it from tauri-build; tauri's own build.rs
    // does this only inside the tauri workspace. Test targets of this crate only.
    if target.contains("windows-msvc") {
        embed_manifest_for_tests();
    }
}

fn embed_manifest_for_tests() {
    const MANIFEST: &str = r#"<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity type="win32" name="Microsoft.Windows.Common-Controls" version="6.0.0.0"
        processorArchitecture="*" publicKeyToken="6595b64144ccf1df" language="*" />
    </dependentAssembly>
  </dependency>
</assembly>
"#;
    let path = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("windows-test-manifest.xml");
    std::fs::write(&path, MANIFEST).expect("write windows-test-manifest.xml");
    println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
    println!(
        "cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}",
        path.display()
    );
}

fn compile_reload_bridge(sdk: &str, arch: &str) {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let src = manifest.join("macos").join("WidgetReload.swift");
    let swift_target = format!("{}-apple-macos11.0", arch);

    let obj = out_dir.join("WidgetReload.o");
    let lib = out_dir.join("libwidget_reload.a");

    assert!(
        Command::new("swiftc")
            .args([
                "-parse-as-library",
                "-emit-object",
                "-o",
                obj.to_str().unwrap(),
                src.to_str().unwrap(),
                "-sdk",
                sdk,
                "-target",
                &swift_target,
                "-O",
                "-whole-module-optimization",
            ])
            .status()
            .expect("swiftc not found — is Xcode installed?")
            .success(),
        "WidgetReload.swift compilation failed"
    );

    assert!(
        Command::new("ar")
            .args(["rcs", lib.to_str().unwrap(), obj.to_str().unwrap()])
            .status()
            .expect("ar failed")
            .success(),
        "ar failed"
    );

    if let Some(p) = swift_stdlib_search_path() {
        println!("cargo:rustc-link-search=native={}", p.display());
    }
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=widget_reload");
    println!("cargo:rustc-link-arg=-weak_framework");
    println!("cargo:rustc-link-arg=WidgetKit");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rerun-if-changed=macos/WidgetReload.swift");
}

fn macos_sdk_path() -> String {
    String::from_utf8(
        Command::new("xcrun")
            .args(["--show-sdk-path", "--sdk", "macosx"])
            .output()
            .expect("xcrun failed")
            .stdout,
    )
    .unwrap()
    .trim()
    .into()
}

fn swift_stdlib_search_path() -> Option<PathBuf> {
    let out = Command::new("xcrun")
        .args(["--toolchain", "default", "--find", "swift-stdlib-tool"])
        .output()
        .ok()?;
    let p = PathBuf::from(String::from_utf8(out.stdout).ok()?.trim().to_string())
        .parent()?
        .parent()?
        .join("lib/swift/macosx");
    p.exists().then_some(p)
}
