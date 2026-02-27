// swift-tools-version:5.9

import PackageDescription

let package = Package(
    name: "TauriWidgets",
    platforms: [
        .iOS(.v17),
        .macOS(.v14),
    ],
    products: [
        .library(name: "TauriWidgets", targets: ["TauriWidgets"]),
    ],
    targets: [
        .target(name: "TauriWidgets", path: "Sources/TauriWidgets"),
    ]
)
