use anyhow::{bail, Context, Result};
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select};
use indicatif::{ProgressBar, ProgressStyle};
use ps_core::PixelRect;
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

// ... [Snowplow Algorithm / extract_rects_optimized] ...
// (Keep the exact same implementation of extract_rects_optimized as you have now)
fn extract_rects_optimized(
    buffer: &[u8],
    width: u32,
    height: u32,
    threshold: u8,
) -> Vec<PixelRect> {
    // ... paste implementation ...
    let w = width as usize;
    let h = height as usize;
    let mut boxes: Vec<PixelRect> = Vec::with_capacity(2000);
    let mut active_indices: Vec<isize> = vec![-1; w];

    for y in 0..h {
        let row_start = y * w;
        let row = &buffer[row_start..row_start + w];
        let mut x = 0;
        while x < w {
            if row[x] < threshold {
                if active_indices[x] != -1 {
                    active_indices[x] = -1;
                }
                x += 1;
                continue;
            }
            let start_x = x;
            while x < w && row[x] >= threshold {
                x += 1;
            }
            let run_width = (x - start_x) as u16;
            let current_start_x = start_x as u16;
            let active_idx = active_indices[start_x];
            let mut merged = false;
            if active_idx != -1 {
                let idx = active_idx as usize;
                if idx < boxes.len() {
                    let b = &mut boxes[idx];
                    if b.y + b.h == (y as u16) && b.x == current_start_x && b.w == run_width {
                        b.h += 1;
                        merged = true;
                    }
                }
            }
            if !merged {
                let new_idx = boxes.len() as isize;
                boxes.push(PixelRect {
                    x: current_start_x,
                    y: y as u16,
                    w: run_width,
                    h: 1,
                });
                active_indices[start_x] = new_idx;
            }
        }
    }
    boxes
}

pub struct ConvertArgs {
    pub project_name: Option<String>,
    pub resolutions: Option<String>,
    pub use_gpu: bool,
}

pub fn run(args: ConvertArgs) -> Result<()> {
    // Setup Paths (Relative to EXE)
    let current_exe = env::current_exe().context("Failed to get exe path")?;
    let exe_dir = current_exe
        .parent()
        .context("Failed to get exe directory")?;
    let assets_root = exe_dir.join("assets");

    // 1. Resolve Project Name
    let project_name = match args.project_name {
        Some(name) => name,
        None => {
            if !assets_root.exists() {
                bail!(
                    "'assets' folder not found at {:?}. Run 'Download' first.",
                    assets_root
                );
            }

            let entries = fs::read_dir(&assets_root)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect::<Vec<_>>();

            if entries.is_empty() {
                bail!("No projects found in 'assets/'.");
            }

            let idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select Project to Convert")
                .items(&entries)
                .default(0)
                .interact()?;

            entries[idx].clone()
        }
    };

    // 2. Resolve Resolutions
    let resolution_list: Vec<String> = match args.resolutions {
        Some(s) => s.split(',').map(|x| x.trim().to_string()).collect(),
        None => {
            let options = vec!["720p", "1080p", "1440p", "2160p"];
            let defaults = vec![true, true, false, false];
            let selections = MultiSelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Select Output Resolutions")
                .items(&options)
                .defaults(&defaults)
                .interact()?;
            if selections.is_empty() {
                bail!("Select at least one resolution.");
            }
            selections.iter().map(|&i| options[i].to_string()).collect()
        }
    };

    // 3. Setup Project Paths
    let project_dir = assets_root.join(&project_name);
    if !project_dir.exists() {
        bail!("Project folder not found: {:?}", project_dir);
    }

    let extensions = ["mkv", "mp4", "avi", "mov", "webm"];
    let vid_path = extensions
        .iter()
        .map(|ext| project_dir.join(format!("{}.{}", project_name, ext)))
        .find(|p| p.exists())
        .with_context(|| format!("No video file found in {:?}", project_dir))?;

    let fps = detect_fps(&vid_path).unwrap_or(30);
    println!("Detected FPS: {}", fps);

    // 4. Processing Loop
    for res_name in resolution_list {
        let target_width = match res_name.as_str() {
            "720p" => 1280,
            "1080p" => 1920,
            "1440p" => 2560,
            "2160p" => 3840,
            _ => {
                println!("Skipping unknown: {}", res_name);
                continue;
            }
        };
        let target_height = target_width * 9 / 16;
        let out_path = project_dir.join(format!("{}_{}.bin", project_name, res_name));

        println!(
            "Processing {} ({}x{})...",
            res_name, target_width, target_height
        );
        process_single_variant(
            &vid_path,
            &out_path,
            target_width,
            target_height,
            fps,
            args.use_gpu,
        )?;
    }

    println!("\n--- SUCCESS: Conversion Complete ---");
    Ok(())
}

fn process_single_variant(
    input: &Path,
    output: &Path,
    width: u32,
    height: u32,
    fps: u16,
    use_gpu: bool,
) -> Result<()> {
    let filters = format!(
        "scale={w}:{h},format=gray,gblur=sigma=1.0:steps=1,eq=contrast=1000:saturation=0",
        w = width,
        h = height
    );
    let mut cmd = Command::new("ffmpeg");
    if use_gpu {
        cmd.arg("-hwaccel").arg("cuda");
    }

    cmd.arg("-i")
        .arg(input)
        .arg("-vf")
        .arg(filters)
        .arg("-f")
        .arg("rawvideo")
        .arg("-pix_fmt")
        .arg("gray")
        .arg("-");

    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to spawn ffmpeg")?;
    let mut stdout = child.stdout.take().context("Failed to open stdout")?;
    let mut file_out = BufWriter::new(File::create(output)?);

    file_out.write_all(&fps.to_le_bytes())?;

    let frame_size = (width * height) as usize;
    let mut buffer = vec![0u8; frame_size];
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} Frames: {pos}")
            .unwrap(),
    );

    let mut frame_count = 0;
    while stdout.read_exact(&mut buffer).is_ok() {
        let rects = extract_rects_optimized(&buffer, width, height, 127);

        let rect_bytes = unsafe {
            std::slice::from_raw_parts(
                rects.as_ptr() as *const u8,
                rects.len() * std::mem::size_of::<PixelRect>(),
            )
        };
        file_out.write_all(rect_bytes)?;

        let eos = PixelRect::EOS_MARKER;
        let eos_bytes = unsafe {
            std::slice::from_raw_parts(
                &eos as *const PixelRect as *const u8,
                std::mem::size_of::<PixelRect>(),
            )
        };
        file_out.write_all(eos_bytes)?;

        frame_count += 1;
        if frame_count % 60 == 0 {
            pb.set_position(frame_count);
        }
    }
    pb.finish_with_message(format!("Done! {} frames.", frame_count));
    Ok(())
}

fn detect_fps(path: &Path) -> Option<u16> {
    let output = Command::new("ffprobe")
        .args(&[
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=r_frame_rate",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(path)
        .output()
        .ok()?;
    let out_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if out_str.contains('/') {
        let parts: Vec<&str> = out_str.split('/').collect();
        let num: f64 = parts[0].parse().ok()?;
        let den: f64 = parts[1].parse().ok()?;
        Some((num / den).round() as u16)
    } else {
        out_str.parse::<f64>().ok().map(|f| f.round() as u16)
    }
}
