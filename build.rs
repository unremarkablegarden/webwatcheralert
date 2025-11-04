fn main() {
    // Only apply these settings on macOS
    #[cfg(target_os = "macos")]
    {
        // Link against AppKit framework (needed for NSImage)
        println!("cargo:rustc-link-lib=framework=AppKit");
        // Link against ApplicationServices (needed for LSCopyApplicationURLsForBundleIdentifier)
        println!("cargo:rustc-link-lib=framework=ApplicationServices");
    }
}
