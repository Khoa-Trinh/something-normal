use anyhow::{bail, Result};
use dialoguer::{theme::ColorfulTheme, Select};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

pub struct RunArgs {
    pub target: Option<String>,
}

pub fn run(args: RunArgs) -> Result<()> {
    // Setup Paths (Relative to EXE)
    let current_exe = env::current_exe()?;
    let exe_dir = current_exe.parent().unwrap();
    let dist_dir = exe_dir.join("dist");

    if !dist_dir.exists() {
        bail!(
            "'dist' folder not found at {:?}. Please run 'build' first.",
            dist_dir
        );
    }

    // 2. Scan for Executables
    let mut executables: Vec<(String, PathBuf)> = fs::read_dir(dist_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let p = e.path();
            p.is_file() && (p.extension().map_or(false, |ext| ext == "exe") || cfg!(unix))
        })
        .map(|e| (e.file_name().to_string_lossy().to_string(), e.path()))
        .collect();

    executables.sort_by(|a, b| a.0.cmp(&b.0));

    if executables.is_empty() {
        bail!("No executables found in 'dist/'.");
    }

    // 3. Resolve Target
    let selected_path = match args.target {
        Some(name) => {
            let found: Vec<&PathBuf> = executables
                .iter()
                .filter(|(f, _)| f.contains(&name))
                .map(|(_, p)| p)
                .collect();
            match found.len() {
                0 => bail!("No executable matching '{}' found.", name),
                1 => found[0].clone(),
                _ => {
                    println!("Multiple matches found for '{}':", name);
                    let options: Vec<String> = executables
                        .iter()
                        .filter(|(f, _)| f.contains(&name))
                        .map(|(f, _)| f.clone())
                        .collect();
                    let idx = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Select version")
                        .items(&options)
                        .default(0)
                        .interact()?;
                    executables
                        .iter()
                        .find(|(n, _)| n == &options[idx])
                        .unwrap()
                        .1
                        .clone()
                }
            }
        }
        None => {
            let options: Vec<String> = executables.iter().map(|(n, _)| n.clone()).collect();
            let idx = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select Executable to Run")
                .items(&options)
                .default(0)
                .interact()?;
            executables[idx].1.clone()
        }
    };

    run_watchdog(&selected_path)
}

fn run_watchdog(exe_path: &Path) -> Result<()> {
    let exe_name = exe_path.file_name().unwrap().to_string_lossy();
    println!("✅ Target Acquired: {}", exe_name);
    println!("🛡️  Watchdog Active. Press Ctrl+C to stop.");

    let mut restart_count = 0;
    loop {
        println!(
            "\n[Watchdog] Starting {} (Instance #{})",
            exe_name,
            restart_count + 1
        );
        let mut child = Command::new(exe_path).spawn()?;
        let exit_status = child.wait()?;
        println!(
            "⚠️  Process exited with code: {}",
            exit_status.code().unwrap_or(-1)
        );
        println!("[Watchdog] Restarting in 2 seconds...");
        thread::sleep(Duration::from_secs(2));
        restart_count += 1;
    }
}
