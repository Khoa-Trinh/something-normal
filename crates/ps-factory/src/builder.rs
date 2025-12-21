use anyhow::{bail, Context, Result};
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::mem;
use std::path::PathBuf;
use std::slice;

// 1. REMOVED the compile-time include!
// const RUNNER_BYTES: &[u8] = include_bytes!("ps-runner.exe");

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct PayloadFooter {
    video_offset: u64,
    video_len: u64,
    audio_offset: u64,
    audio_len: u64,
    width: u16,
    height: u16,
    magic: [u8; 8],
}

#[derive(Debug, Clone)]
pub struct BuildTarget {
    pub project: String,
    pub resolution: String,
    pub width: u16,
    pub height: u16,
    pub bin_path: PathBuf,
    pub audio_path: PathBuf,
}

pub struct BuildArgs {
    pub project_name: Option<String>,
    pub resolutions: Option<String>,
    pub build_all: bool,
}

pub fn run(args: BuildArgs) -> Result<()> {
    // =========================================================
    // 1. SETUP PATHS (Relative to the running CLI.exe)
    // =========================================================
    let current_exe = env::current_exe().context("Failed to get exe path")?;
    let exe_dir = current_exe
        .parent()
        .context("Failed to get exe directory")?;

    let assets_dir = exe_dir.join("assets");
    let dist_dir = exe_dir.join("dist");

    // CRITICAL CHANGE: Look for the template next to the CLI
    let template_path = exe_dir.join("ps-runner.exe");

    // =========================================================
    // 2. CHECK & LOAD TEMPLATE
    // =========================================================
    if !template_path.exists() {
        bail!(
            "❌ Missing Template!\nCould not find 'ps-runner.exe' at:\n{:?}\n\nPlease build ps-runner first and copy it next to this CLI.",
            template_path
        );
    }

    println!("📄 Loading template from: {:?}", template_path);
    let template_bytes =
        fs::read(&template_path).context("Failed to read ps-runner.exe template")?;

    // =========================================================
    // 3. SCAN & SELECT TARGETS
    // =========================================================
    let all_targets = get_available_builds(&assets_dir)?;
    if all_targets.is_empty() {
        bail!(
            "No valid assets found in {:?}. Run 'convert' command first!",
            assets_dir
        );
    }

    let selected_targets: Vec<BuildTarget> =
        if args.build_all || args.project_name.is_some() || args.resolutions.is_some() {
            println!("Filtering targets based on flags...");
            let req_res: Option<Vec<String>> = args
                .resolutions
                .as_ref()
                .map(|s| s.split(',').map(|r| r.trim().to_string()).collect());
            all_targets
                .into_iter()
                .filter(|t| {
                    if let Some(p) = &args.project_name {
                        if &t.project != p {
                            return false;
                        }
                    }
                    if let Some(r_list) = &req_res {
                        if !r_list.contains(&t.resolution) {
                            return false;
                        }
                    }
                    true
                })
                .collect()
        } else {
            let options: Vec<String> = all_targets
                .iter()
                .map(|t| format!("{} @ {}", t.project, t.resolution))
                .collect();
            let selection = MultiSelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Select Targets")
                .items(&options)
                .interact()?;
            if selection.is_empty() {
                println!("Nothing selected.");
                return Ok(());
            }
            selection
                .into_iter()
                .map(|i| all_targets[i].clone())
                .collect()
        };

    if selected_targets.is_empty() {
        bail!("No matching build targets found.");
    }

    fs::create_dir_all(&dist_dir)?;
    println!("Selected {} target(s).", selected_targets.len());
    println!("📂 Dist directory: {:?}", dist_dir);

    // =========================================================
    // 4. PATCHING LOOP
    // =========================================================
    for target in selected_targets {
        let exe_name = format!("{}_{}.exe", target.project, target.resolution);
        let output_path = dist_dir.join(&exe_name);
        println!("📦 Patching {}...", exe_name);

        let video_data = fs::read(&target.bin_path).context("Failed to read video bin")?;
        let audio_data = fs::read(&target.audio_path).context("Failed to read audio ogg")?;

        let template_len = template_bytes.len() as u64;
        let video_len = video_data.len() as u64;
        let audio_len = audio_data.len() as u64;

        let video_offset = template_len;
        let audio_offset = template_len + video_len;

        let footer = PayloadFooter {
            video_offset,
            video_len,
            audio_offset,
            audio_len,
            width: target.width,
            height: target.height,
            magic: *b"PS_PATCH",
        };

        let mut file = File::create(&output_path)?;

        // 1. Write the Loaded Template
        file.write_all(&template_bytes)?;

        // 2. Append Assets
        file.write_all(&video_data)?;
        file.write_all(&audio_data)?;

        // 3. Append Footer
        let footer_bytes = unsafe {
            slice::from_raw_parts(
                &footer as *const PayloadFooter as *const u8,
                mem::size_of::<PayloadFooter>(),
            )
        };
        file.write_all(footer_bytes)?;

        println!("   ✨ Created: {:?}", output_path);
    }
    println!("\n✅ All Builds Complete!");
    Ok(())
}

fn get_available_builds(assets_dir: &std::path::Path) -> Result<Vec<BuildTarget>> {
    if !assets_dir.exists() {
        return Ok(vec![]);
    }
    let mut targets = Vec::new();
    let resolutions = vec![
        ("720p", 1280, 720),
        ("1080p", 1920, 1080),
        ("1440p", 2560, 1440),
        ("2160p", 3840, 2160),
    ];

    for entry in fs::read_dir(assets_dir)? {
        let entry = entry?;
        if entry.path().is_dir() {
            let project_name = entry.file_name().to_string_lossy().to_string();
            let project_dir = entry.path();
            let audio_path = project_dir.join(format!("{}.ogg", project_name));
            if !audio_path.exists() {
                continue;
            }

            for (res_name, w, h) in &resolutions {
                let bin_name = format!("{}_{}.bin", project_name, res_name);
                let bin_path = project_dir.join(&bin_name);
                if bin_path.exists() {
                    targets.push(BuildTarget {
                        project: project_name.clone(),
                        resolution: res_name.to_string(),
                        width: *w,
                        height: *h,
                        bin_path: fs::canonicalize(&bin_path)?,
                        audio_path: fs::canonicalize(&audio_path)?,
                    });
                }
            }
        }
    }
    Ok(targets)
}
