# ✨ Pixel Shell

**High-Performance Desktop Overlay Engine & Asset Factory**

Pixel Shell is a specialized engine designed to render high-framerate, transparent video overlays on Windows with minimal resource usage. It utilizes a custom **"Snowplow" RLE compression algorithm** to render uncompressed video frames directly via GDI, bypassing standard video players for absolute background transparency.

The project features a unique **Binary Patching Architecture**: instead of compiling code for every video, the CLI injects compressed asset data directly into a pre-compiled generic **Runner** executable, creating standalone, portable `.exe` files instantly.

---

## 📥 Download Pre-built Binaries

Don’t want to build from source?

You can download the latest ready-to-use versions of the CLI Factory and the Runner Template directly from **GitHub Releases**.

- Download **ps-cli.exe** (The Builder Tool)
- Download **ps-runner.exe** (The Template)

Place them in the same folder, and you are ready to go.

---

## 🚀 Features

- ⚡ **Zero-Copy Rendering** — Custom `.bin` format optimized for CPU-based sparse rendering
- 🔊 **Audio Sync** — High-priority audio thread using `kira` for precise A/V synchronization
- 📦 **Standalone Output** — Generates single-file `.exe` overlays with no external dependencies
- 🛠️ **All-in-One CLI** — Download, Convert, Debug, and Build in one tool
- 🛡️ **Watchdog Mode** — Automatically restarts overlays if they crash or are closed

---

## 📂 Project Structure

This is a Cargo workspace organized into applications and shared libraries.

```text
pixel-shell/
├── .github/workflows/   # CI/CD pipelines
├── apps/
│   ├── ps-cli/          # Command Line Interface (user tool)
│   ├── ps-gui/          # Experimental GUI frontend
│   └── ps-runner/       # Template EXE (player engine)
├── crates/
│   ├── ps-core/         # Shared data structures (PixelRect, headers)
│   └── ps-factory/      # Binary building & patching logic
├── target/              # Build artifacts
├── pixel-shell.ico      # Application icon
└── Cargo.toml           # Workspace configuration
```

---

## 🛠️ Building from Source

If you want to contribute or modify the engine, follow these steps.

### Prerequisites

- Rust (via Rustup)
- FFmpeg & FFprobe (required for asset conversion)
- yt-dlp (required for downloading source material)

### Compilation

Since the Factory CLI relies on the Runner template existing at runtime, you must build both.

```bash
git clone https://github.com/Khoa-Trinh/PixelShell.git
cd PixelShell
cargo build --release
```

### Assemble the Toolset

Create a working folder (e.g., `PixelShellTool`) and copy the artifacts:

- `target/release/ps-cli.exe` -> `PixelShellTool/ps-cli.exe`
- `target/release/ps-runner.exe` -> `PixelShellTool/ps-runner.exe`

---

## 🎮 CLI Usage Guide

Open a terminal in the folder containing the executables.

### 1. Download Content

Downloads a video, extracts audio, and prepares it for processing.

```bash
ps-cli.exe (with interative prompts)
# or using direct arguments
ps-cli.exe download --url "https://youtu.be/..." --resolution 1080p --project "my_overlay"
```

### 2. Convert Assets

Transcodes video frames into the optimized `.bin` format using the Snowplow algorithm.

```bash
ps-cli.exe convert (with interative prompts)
# or using direct arguments
ps-cli.exe convert --project "my_overlay" --resolutions "1080p,720p" --use-gpu
```

### 3. Build Standalone EXE

Injects converted assets into the runner template.

```bash
ps-cli.exe build (with interative prompts)
# or using direct arguments
ps-cli.exe build --project "my_overlay" --resolutions "1080p,720p"
# Output will be placed in the /dist folder
```

### 4. Run the Overlay

Run via command line instead of double-clicking the exe will enable Watchdog mode.

```bash
ps-cli.exe run (with interative prompts)
# or using direct arguments
ps-cli.exe run --target "my_overlay_1080p.exe" (name of the generated exe)
# Output will be placed in the /dist folder
```

---

## 🔧 Troubleshooting

- **Template not found** — Ensure `ps-runner.exe` is in the same folder as the CLI executable
- **FFmpeg not found** — Run `ffmpeg -version` and verify your PATH configuration

---

## 📜 License

This project is licensed under the **MIT License**.
