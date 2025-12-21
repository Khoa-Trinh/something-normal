use anyhow::{bail, Context, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use dialoguer::{theme::ColorfulTheme, Select};
use minifb::{Key, Window, WindowOptions};
use std::env;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;
use std::time::{Duration, Instant};

pub struct DebugArgs {
    pub project_name: Option<String>,
    pub file_name: Option<String>,
}

pub fn run(args: DebugArgs) -> Result<()> {
    // Setup Paths (Relative to EXE)
    let current_exe = env::current_exe().context("Failed to get exe path")?;
    let exe_dir = current_exe
        .parent()
        .context("Failed to get exe directory")?;
    let assets_dir = exe_dir.join("assets");

    if !assets_dir.exists() {
        bail!(
            "Assets folder not found at {:?}. Please run 'Download' first.",
            assets_dir
        );
    }

    // --- STEP 1: RESOLVE PROJECT ---
    let project_path = match args.project_name {
        Some(name) => {
            let p = assets_dir.join(&name);
            if !p.exists() {
                bail!("Project '{}' not found.", name);
            }
            p
        }
        None => {
            let mut projects: Vec<_> = fs::read_dir(&assets_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .collect();
            projects.sort_by_key(|e| e.file_name());

            if projects.is_empty() {
                bail!("No projects found in assets folder.");
            }

            let project_names: Vec<String> = projects
                .iter()
                .map(|p| p.file_name().to_string_lossy().to_string())
                .collect();

            let p_idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select Project")
                .default(0)
                .items(&project_names)
                .interact()?;
            projects[p_idx].path()
        }
    };

    // --- STEP 2: RESOLVE BIN FILE ---
    let bin_path = match args.file_name {
        Some(name) => {
            let p = project_path.join(&name);
            let p = if p.exists() {
                p
            } else {
                project_path.join(format!("{}.bin", name))
            };
            if !p.exists() {
                bail!("File '{}' not found.", name);
            }
            p
        }
        None => {
            let mut bin_files: Vec<_> = fs::read_dir(&project_path)?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("bin"))
                .collect();
            bin_files.sort_by_key(|e| e.file_name());

            if bin_files.is_empty() {
                bail!("No .bin files found in {:?}", project_path);
            }

            let bin_names: Vec<String> = bin_files
                .iter()
                .map(|b| b.file_name().to_string_lossy().to_string())
                .collect();

            let b_idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select Bin File")
                .default(0)
                .items(&bin_names)
                .interact()?;
            bin_files[b_idx].path()
        }
    };

    play_bin_file(&bin_path)
}

fn play_bin_file(path: &Path) -> Result<()> {
    println!("Analyzing: {:?}...", path);
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);
    let fps = reader.read_u16::<LittleEndian>()?;
    println!("Target FPS: {}", fps);

    let width = 1920;
    let height = 1080;
    let mut window = Window::new(
        &format!("Debug View - {} FPS", fps),
        width,
        height,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .context("Unable to create window")?;

    let mut buffer: Vec<u32> = vec![0; width * height];
    let frame_duration = Duration::from_secs_f64(1.0 / fps as f64);
    let mut frame_idx = 0;
    let mut rect_buf = [0u8; 8];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let start_time = Instant::now();
        buffer.fill(0xFF000000);
        let mut rect_count = 0;

        loop {
            if reader.read_exact(&mut rect_buf).is_err() {
                return Ok(());
            }
            let x = u16::from_le_bytes(rect_buf[0..2].try_into().unwrap());
            let y = u16::from_le_bytes(rect_buf[2..4].try_into().unwrap());
            let w = u16::from_le_bytes(rect_buf[4..6].try_into().unwrap());
            let h = u16::from_le_bytes(rect_buf[6..8].try_into().unwrap());
            if w == 0 && h == 0 {
                break;
            }
            rect_count += 1;
            draw_rect(
                &mut buffer,
                width,
                height,
                x as usize,
                y as usize,
                w as usize,
                h as usize,
            );
        }

        if frame_idx % fps as usize == 0 {
            print!("\rFrame: {} | Rects: {}    ", frame_idx, rect_count);
            use std::io::Write;
            std::io::stdout().flush().ok();
        }

        window.update_with_buffer(&buffer, width, height)?;
        frame_idx += 1;
        let elapsed = start_time.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
    Ok(())
}

fn draw_rect(
    buffer: &mut [u32],
    screen_w: usize,
    screen_h: usize,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
) {
    let right = (x + w).min(screen_w);
    let bottom = (y + h).min(screen_h);
    for r in y..bottom {
        let row_start = r * screen_w;
        if right > x {
            buffer[row_start + x..row_start + right].fill(0xFFFFFFFF);
        }
    }
}
