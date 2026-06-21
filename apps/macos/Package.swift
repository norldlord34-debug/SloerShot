// swift-tools-version:5.9
import PackageDescription

// SloerShot macOS app. Build the native core first: cargo build -p shotcore --release
// then: swift build (or open in Xcode for a full .app bundle).
let package = Package(
    name: "SloerShot",
    platforms: [.macOS(.v14)],
    targets: [
        .systemLibrary(name: "CShotCore", path: "Sources/CShotCore"),
        .executableTarget(
            name: "SloerShot",
            dependencies: ["CShotCore"],
            // Link against the Rust core: build it first with cargo build -p shotcore --release.
 // The debug path is a fallback for a plain cargo build -p shotcore. At runtime the
 // dylib must be loadable: bundle libshotcore.dylib into the .app Frameworks, or set
 // DYLD_LIBRARY_PATH to the matching target dir for swift run during development.
 linkerSettings: [.unsafeFlags(["-L../../target/release", "-L../../target/debug"])]
        )
    ]
)
