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
    dependencies: [
        .package(url: "https://github.com/pointfreeco/swift-snapshot-testing", from: "1.17.0"),
    ],
    targets: [
        .target(name: "TauriWidgets", path: "Sources/TauriWidgets"),
        .testTarget(
            name: "TauriWidgetsTests",
            dependencies: [
                "TauriWidgets",
                .product(name: "SnapshotTesting", package: "swift-snapshot-testing"),
            ],
            path: "Tests/TauriWidgetsTests"
        ),
    ]
)
