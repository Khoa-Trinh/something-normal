# Pixel Shell

**Pixel Shell** is a high-performance, transparent desktop overlay engine written in Rust. It is designed to play silhouette-style animations (like _Bad Apple!!_) directly on your Windows desktop at 60FPS+ with zero background interference.

Unlike standard video players, Pixel Shell uses a custom **Sparse Binary Format (.bin)** to render only the active pixels, allowing it to bypass standard window composition limitations and "flicker" issues.

---

## 🚀 Key Features

### 🦀 Rust Renderer (The Engine)

- **Direct Memory Access (GDI):** Uses `CreateDIBSection` to write pixels directly to a raw buffer in RAM, bypassing thousands of slow GDI system calls.
- **Atomic Updates:** Utilizes `UpdateLayeredWindow` with Alpha Blending for tear-free, VSync-locked rendering.
- **Zero-Copy Logic:** Efficiently parses binary frame data with almost zero CPU overhead by iterating through raw coordinate slices.
- **Kira Audio:** Integrated low-latency audio synchronization with high-precision clocking.

### 🐍 Python Pipeline (The Factory)

- **Smart Downloader:** Preprocesses videos using `yt-dlp` and `ffmpeg` with Lanczos Upscaling and High-Contrast Thresholding to create perfect binary source material.
- **"Snowplow" Algorithm:** A custom Numba-optimized RLE extractor that clears stale memory states between rows to prevent vertical artifacting.
- **"Gap Welding":** Automatically fuses horizontal striping artifacts using vertical morphological dilation.
- **GPU Acceleration:** Optional NVIDIA CUDA support (via CuPy) for ultra-fast high-resolution processing.

---

## 🛠️ Prerequisites

- **Rust:** [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
- **Python 3.10+:** [https://www.python.org/downloads/](https://www.python.org/downloads/)
- **FFmpeg:** Must be installed and added to your system `PATH`.
- **Visual Studio Build Tools:** Required for the `windows-rs` crate (Select **Desktop development with C++**).

---

## 📦 Installation

### 1. Clone the Repository

```bash
git clone https://github.com/your-username/pixel-shell.git
cd pixel-shell
```

### 2. Install Python Dependencies

```bash
cd compile
pip install -r requirements.txt
```

Optional (NVIDIA GPU acceleration):

```bash
pip install cupy-cuda12x
```

### 3. Build the Rust Project

```bash
cd ..
cargo build --release
```

---

## 🎬 Usage Workflow

### Step 1: Download & Prep Video

This script forces the source into a **Binary-Safe** format (pure black & white, no motion blur).

```bash
cd compile
python download-video.py
```

- Paste your YouTube URL
- Choose resolution (e.g. 1080p)
- FPS Strategy: **[1] 30 FPS (Native)**

> Native frames are sharper and prevent ghosting artifacts in the binary detector.

---

### Step 2: Convert to BIN

Analyzes frames and compiles them into the sparse `.bin` coordinate format.

```bash
python video-to-bin.py
```

- Select project folder
- Choose **CPU** or **GPU** (if available)

Output:

```
assets/<project_name>/<project_name>.bin
```

---

### Step 3: Run the Overlay

Update the `.bin` path in `main.rs` or `build.py`, then run:

```bash
cd ..
cargo run --release
```

---

## 📂 Project Structure

```
pixel-shell/
├── assets/               # Raw videos & generated .bin files
├── compile/              # Python Processing Pipeline
│   ├── download-video.py # High-Contrast FFmpeg Preprocessor
│   ├── video-to-bin.py   # Sparse Coordinate Converter
│   ├── helpers.py        # Snowplow & Gap Welding logic
│   └── debug_bin.py      # Bin visualizer / inspector
├── src/                  # Rust Renderer Source
│   ├── main.rs           # GDI rendering & window logic
│   ├── window.rs         # Shared constants
│   └── audio.rs          # Kira audio implementation
├── Cargo.toml            # Rust dependency manifest
└── build.py              # Automation script
```

---

## 🔧 Troubleshooting

**Q: The video is flickering or stuttering**

- Ensure you are running with `--release`
- Renderer must stay under **16.6ms per frame**
- On multi-monitor setups, size the window to `SM_CXSCREEN` instead of the virtual screen

**Q: Horizontal stripes or gaps appear**

- Confirm you are using the **Snowplow** version of `helpers.py`
- Ensure `active_boxes` is cleared per row
- Verify **Gap Welding** is enabled

**Q: Red snow / noise pixels**

- Source video contains compression artifacts
- Re-run `download-video.py` with high-contrast filtering enabled

---

## 📜 License

MIT License
