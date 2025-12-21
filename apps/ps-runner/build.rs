use std::env;
use std::path::Path;

fn main() {
    // 1. Only run this on Windows
    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() != "windows" {
        return;
    }

    let mut res = winres::WindowsResource::new();

    // 2. Icon Path
    // winres resolves paths relative to Cargo.toml (apps/ps-runner/Cargo.toml)
    // So we look up two levels to the workspace root's assets folder.
    let icon_path = "../../pixel-shell.ico";

    // Optional: Warn if icon is missing instead of failing silently
    if !Path::new(icon_path).exists() {
        println!("cargo:warning=⚠️ Icon not found at: {}", icon_path);
    } else {
        res.set_icon(icon_path);
    }

    // 3. Metadata
    res.set(
        "FileDescription",
        "Pixel Shell High-Performance Overlay Engine",
    );
    res.set("ProductName", "Pixel Shell");
    res.set("CompanyName", "ShineeKun");
    res.set("FileVersion", "1.0.0.0");
    res.set("ProductVersion", "1.0.0.0");
    res.set(
        "LegalCopyright",
        "Copyright © 2025 ShineeKun. All Rights Reserved.",
    );
    res.set("OriginalFilename", "runner.exe");

    // 4. Manifest (Critical for High DPI / 4K monitors)
    res.set_manifest(r#"
        <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
        <application xmlns="urn:schemas-microsoft-com:asm.v3">
            <windowsSettings>
                <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
                <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
            </windowsSettings>
        </application>
        <dependency>
            <dependentAssembly>
                <assemblyIdentity
                    type="win32"
                    name="Microsoft.Windows.Common-Controls"
                    version="6.0.0.0"
                    processorArchitecture="*"
                    publicKeyToken="6595b64144ccf1df"
                    language="*"
                />
            </dependentAssembly>
        </dependency>
        </assembly>
    "#);

    // 5. Compile
    if let Err(e) = res.compile() {
        panic!("Resource Compile Error: {}", e);
    }
}
