// swift-tools-version:5.3

import PackageDescription

let package = Package(
    name: "tauri-plugin-widgets",
    platforms: [
        .macOS(.v10_13),
        .iOS(.v13),
    ],
    products: [
        .library(
            name: "tauri-plugin-widgets",
            type: .static,
            targets: ["tauri-plugin-widgets"]
        )
    ],
    dependencies: [
        .package(name: "Tauri", path: "../.tauri/tauri-api")
    ],
    targets: [
        .target(
            name: "tauri-plugin-widgets",
            dependencies: [
                .byName(name: "Tauri")
            ],
            path: "Sources"
        ),
        .testTarget(
            name: "tauri-plugin-widgets-tests",
            dependencies: [
                "tauri-plugin-widgets"
            ],
            path: "Tests/PluginTests"
        )
    ]
)
