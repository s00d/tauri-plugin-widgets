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
    "get_widget_config",
    "widget_action",
    "poll_pending_actions",
];

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();

    let result = tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .try_build();

    if !(cfg!(docsrs) && target.contains("android")) {
        result.unwrap();
    }

    if target.contains("apple-darwin") {
        let sdk = macos_sdk_path();
        let arch = if target.contains("aarch64") {
            "arm64"
        } else {
            "x86_64"
        };
        compile_reload_bridge(&sdk, arch);
    }
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
