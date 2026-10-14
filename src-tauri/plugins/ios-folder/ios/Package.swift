// swift-tools-version:5.3
import PackageDescription

let package = Package(
  name: "tauri-plugin-ios-folder",
  platforms: [
    .macOS(.v10_13),
    .iOS(.v13),
  ],
  products: [
    .library(
      name: "tauri-plugin-ios-folder",
      type: .static,
      targets: ["tauri-plugin-ios-folder"])
  ],
  dependencies: [
    .package(name: "Tauri", path: "../.tauri/tauri-api")
  ],
  targets: [
    .target(
      name: "tauri-plugin-ios-folder",
      dependencies: [
        .byName(name: "Tauri")
      ],
      path: "Sources")
  ]
)
