//! Emit Swift / Kotlin wire catalogs from the Rust IR SoT.

use std::env;
use std::fs;
use std::path::PathBuf;
use tauri_plugin_widgets::codegen::{
    emit_widget_element_swift, emit_wire_catalog_kotlin, emit_wire_catalog_swift,
    emit_wire_fields_kotlin,
};

fn main() {
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let swift_out = root.join("swift/Sources/TauriWidgets/Generated/WireCatalog.swift");
    fs::create_dir_all(swift_out.parent().unwrap()).unwrap();
    fs::write(&swift_out, emit_wire_catalog_swift()).expect("write WireCatalog.swift");
    eprintln!("wrote {}", swift_out.display());

    let el_out = root.join("swift/Sources/TauriWidgets/Generated/WidgetElement.swift");
    fs::write(&el_out, emit_widget_element_swift()).expect("write WidgetElement.swift");
    eprintln!("wrote {}", el_out.display());

    let fields_out = root.join("android/src/main/java/render/WireFields.kt");
    fs::write(&fields_out, emit_wire_fields_kotlin()).expect("write WireFields.kt");
    eprintln!("wrote {}", fields_out.display());

    let kt_out = root.join("android/src/main/java/render/WireTypes.kt");
    fs::create_dir_all(kt_out.parent().unwrap()).unwrap();
    fs::write(&kt_out, emit_wire_catalog_kotlin()).expect("write WireTypes.kt");
    eprintln!("wrote {}", kt_out.display());
}
