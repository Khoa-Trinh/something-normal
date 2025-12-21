use anyhow::{bail, Context, Result};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

// --- Helper Structs ---

#[derive(Deserialize)]
struct FFProbeOutput {
    streams: Vec<FFProbeStream>,
}

#[derive(Deserialize)]
struct FFProbeStream {
    codec_type: String,
    width: Option<u32>,
    height: Option<u32>,
    avg_frame_rate: String,
}

pub struct DownloadArgs {
    pub url: Option<String>,
    pub resolution: Option<String>,
    pub fps: Option<u32>,
    pub project_name: Option<String>,
}

// --- Main Logic ---

pub fn check_dependencies() -> Result<()> {
    let deps = ["yt-dlp", "ffmpeg", "ffprobe"];
    for dep in deps {
        which::which(dep).with_context(|| format!("Error: '{}' not found in PATH.", dep))?;
    }
    Ok(())
}

fn get_video_info(path: &Path) -> Result<(u32, u32, u32)> {
    let output = Command::new("ffprobe")
        .args(&["-v", "quiet", "-print_format", "json", "-show_streams"])
        .arg(path)
        .output()?;

    let parsed: FFProbeOutput = serde_json::from_slice(&output.stdout)?;

    for stream in parsed.streams {
        if stream.codec_type == "video" {
            let w = stream.width.unwrap_or(0);
            let h = stream.height.unwrap_or(0);
            let fps = if stream.avg_frame_rate.contains('/') {
                let parts: Vec<&str> = stream.avg_frame_rate.split('/').collect();
                let num: f64 = parts[0].parse().unwrap_or(0.0);
                let den: f64 = parts[1].parse().unwrap_or(1.0);
                (num / den).round() as u32
            } else {
                stream.avg_frame_rate.parse::<f64>().unwrap_or(0.0).round() as u32
            };
            return Ok((w, h, fps));
        }
    }
    bail!("No video stream found")
}

pub fn run(args: DownloadArgs) -> Result<()> {
    // 1. Get URL
    let url: String = match args.url {
        Some(u) => u,
        None => Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter YouTube URL")
            .interact_text()?,
    };

    // 2. Select Resolution
    let (tw, th) = match args.resolution.as_deref() {
        Some("720p") => (1280, 720),
        Some("1080p") => (1920, 1080),
        Some("1440p") => (2560, 1440),
        Some("2160p") => (3840, 2160),
        Some(other) => {
            println!(
                "Warning: Unknown resolution '{}', defaulting to 1080p",
                other
            );
            (1920, 1080)
        }
        None => {
            let resolutions = vec!["720p", "1080p", "1440p", "2160p"];
            let res_idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select Target Resolution")
                .default(1)
                .items(&resolutions)
                .interact()?;
            match res_idx {
                0 => (1280, 720),
                1 => (1920, 1080),
                2 => (2560, 1440),
                3 => (3840, 2160),
                _ => (1920, 1080),
            }
        }
    };

    // 3. Select FPS
    let tfps = match args.fps {
        Some(f) => f,
        None => {
            let fps_options = vec!["30 FPS", "60 FPS", "120 FPS", "144 FPS", "165 FPS"];
            let fps_idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select FPS Strategy")
                .default(0)
                .items(&fps_options)
                .interact()?;
            match fps_idx {
                0 => 30,
                1 => 60,
                2 => 120,
                3 => 144,
                4 => 165,
                _ => 30,
            }
        }
    };

    // 4. Project Name
    let project_name: String = match args.project_name {
        Some(name) => name,
        None => Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter Project Name")
            .default("default_project".into())
            .interact_text()?,
    };

    // 5. Setup Paths (Relative to EXE)
    let current_exe = env::current_exe().context("Failed to get exe path")?;
    let exe_dir = current_exe
        .parent()
        .context("Failed to get exe directory")?;
    let assets_root = exe_dir.join("assets");

    let output_dir = assets_root.join(&project_name);
    fs::create_dir_all(&output_dir)?;

    println!("📂 Working directory: {:?}", output_dir);

    let temp_raw = output_dir.join("temp_raw.mp4");
    let final_video = output_dir.join(format!("{}.mkv", project_name));
    let final_audio = output_dir.join(format!("{}.ogg", project_name));

    println!("\n[1/3] Downloading source...");
    let status = Command::new("yt-dlp")
        .args(&[
            "-f",
            "bestvideo+bestaudio/best",
            &url,
            "-o",
            temp_raw.to_str().unwrap(),
            "--merge-output-format",
            "mp4",
        ])
        .status()?;

    if !status.success() {
        bail!("Download failed");
    }

    let (cw, ch, cfps) = get_video_info(&temp_raw)?;
    println!("Source detected: {}x{} @ {} FPS", cw, ch, cfps);

    // 6. Construct FFmpeg Filters
    let mut filters = Vec::new();
    if cw != tw || ch != th {
        filters.push(format!(
            "scale={}:{}:force_original_aspect_ratio=increase:flags=lanczos",
            tw, th
        ));
        filters.push(format!("crop={}:{}", tw, th));
        filters.push("setsar=1".to_string());
    }
    if cfps != tfps {
        filters.push(format!("fps={}", tfps));
    }
    filters.push("format=gray".to_string());
    filters.push("gblur=sigma=1.5:steps=1".to_string());
    filters.push("eq=contrast=1000:saturation=0".to_string());

    let filter_str = filters.join(",");

    println!("\n[2/3] Processing Video ({}x{} @ {} FPS)...", tw, th, tfps);
    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(&temp_raw)
        .arg("-vf")
        .arg(&filter_str)
        .args(&[
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-qp",
            "0",
            "-an",
            "-y",
        ])
        .arg(&final_video)
        .status()?;

    if !status.success() {
        bail!("Video processing failed");
    }

    println!("\n[3/3] Extracting Audio...");
    let status = Command::new("ffmpeg")
        .arg("-i")
        .arg(&temp_raw)
        .args(&["-vn", "-acodec", "libvorbis", "-q:a", "5", "-y"])
        .arg(&final_audio)
        .status()?;

    if !status.success() {
        bail!("Audio extraction failed");
    }

    if temp_raw.exists() {
        fs::remove_file(temp_raw)?;
    }

    println!("\n--- SUCCESS: {} Ready ---", project_name);
    Ok(())
}
