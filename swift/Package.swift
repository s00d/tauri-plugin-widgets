// swift-tools-version:5.9

import Foundation
import PackageDescription

// The npm package ships Sources only. Declaring the test target (and its snapshot-testing
// dependency) without Tests/ makes SwiftPM reject the whole package in user projects.
let testsDir = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
    .appendingPathComponent("Tests/TauriWidgetsTests").path
let hasTests = FileManager.default.fileExists(atPath: testsDir)

let package = Package(
    name: "TauriWidgets",
    platforms: [
        .iOS(.v17),
        .macOS(.v14),
    ],
    products: [
        .library(name: "TauriWidgets", targets: ["TauriWidgets"]),
    ],
    dependencies: hasTests
        ? [.package(url: "https://github.com/pointfreeco/swift-snapshot-testing", from: "1.17.0")]
        : [],
    targets: [
        .target(name: "TauriWidgets", path: "Sources/TauriWidgets"),
    ] + (hasTests
        ? [
            .testTarget(
                name: "TauriWidgetsTests",
                dependencies: [
                    "TauriWidgets",
                    .product(name: "SnapshotTesting", package: "swift-snapshot-testing"),
                ],
                path: "Tests/TauriWidgetsTests"
            ),
        ]
        : [])
)
