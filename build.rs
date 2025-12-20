use std::env;
use std::path::Path;

fn main() {
    if env::var("BIN_PATH").is_err() {
        let default_bin = "../assets/bad_apple/bad_apple_1080p.bin";
        let default_audio = "../assets/bad_apple/bad_apple.ogg";

        println!("cargo:rustc-env=BIN_PATH={}", default_bin);
        println!("cargo:rustc-env=AUDIO_PATH={}", default_audio);
    }

    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();

        res.set_manifest(
            r#"
            <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
            <application xmlns="urn:schemas-microsoft-com:asm.v3">
                <windowsSettings>
                    <dpiAware xmlns="http:
                    <dpiAwareness xmlns="http:
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
            "#,
        );

        if Path::new("../assets/pixel-shell.ico").exists() {
            res.set_icon("../assets/pixel-shell.ico");
        }

        if let Err(e) = res.compile() {
            println!("cargo:warning=Resource compilation failed: {}", e);
        }
    }
}
