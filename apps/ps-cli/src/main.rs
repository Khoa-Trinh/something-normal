use clap::{Parser, Subcommand};
use ps_factory::{builder, converter, debugger, downloader, runner};

#[derive(Parser)]
#[command(name = "Pixel Shell Factory")]
#[command(version = "1.0")]
#[command(about = "All-in-one tool for creating Pixel Shell overlays")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 1. Download & Prep Video (Interactive)
    /// Downloads YouTube video, converts to Lossless MKV, extracts Audio.
    Download {
        /// YouTube URL
        #[arg(short, long)]
        url: Option<String>,

        /// Target Resolution (e.g., "1080p", "1440p")
        #[arg(short, long)]
        resolution: Option<String>,

        /// Target FPS (e.g., 30, 60, 144)
        #[arg(short, long)]
        fps: Option<u32>,

        /// Project folder name
        #[arg(short, long)]
        project: Option<String>,
    },
    /// 2. Convert Video to BIN
    /// Runs the Snowplow algorithm to generate optimized .bin data.
    Convert {
        /// Project folder name (scans 'assets/' if missing)
        #[arg(short, long)]
        project: Option<String>,

        /// Resolutions (e.g. "1080p,720p"). Prompts if missing.
        #[arg(short, long)]
        resolutions: Option<String>,

        /// Force GPU usage (flag only)
        #[arg(long, default_value_t = false)]
        gpu: bool,
    },

    /// 3. Visualize a generated .bin file
    /// Opens a window to play back the .bin file to verify data integrity.
    Debug {
        /// Project folder name
        #[arg(short, long)]
        project: Option<String>,

        /// Specific .bin file name (e.g. "bad_apple_1080p.bin")
        #[arg(short, long)]
        file: Option<String>,
    },

    /// 4. Compile Final Executables (Release)
    /// Compiles the Runner for selected resolutions and moves them to 'dist/'.
    Build {
        /// Filter by Project Name (e.g. "bad_apple")
        #[arg(short, long)]
        project: Option<String>,

        /// Filter by Resolutions (e.g. "1080p,720p")
        #[arg(short, long)]
        resolutions: Option<String>,

        /// Build ALL valid targets found in assets
        #[arg(short, long, default_value_t = false)]
        all: bool,
    },

    /// 5. Run & Watchdog
    /// Launches a built executable and auto-restarts it if it closes/crashes.
    Run {
        /// Name of the target to run (e.g. "bad_apple"). Auto-matches if unique.
        #[arg(short, long)]
        target: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Download {
            url,
            resolution,
            fps,
            project,
        } => {
            println!("--- [1/4] Downloader ---");
            if let Err(e) = downloader::check_dependencies() {
                eprintln!("❌ Dependency Missing: {}", e);
                return;
            }

            // Create a config object to pass to the function
            let args = downloader::DownloadArgs {
                url: url.clone(),
                resolution: resolution.clone(),
                fps: *fps,
                project_name: project.clone(),
            };

            if let Err(e) = downloader::run(args) {
                eprintln!("❌ Download Error: {}", e);
            }
        }

        Commands::Convert {
            project,
            resolutions,
            gpu,
        } => {
            println!("--- [2/4] Converter ---");

            // Create config object with Optional fields
            let args = converter::ConvertArgs {
                project_name: project.clone(),
                resolutions: resolutions.clone(),
                use_gpu: *gpu,
            };

            if let Err(e) = converter::run(args) {
                eprintln!("❌ Conversion Error: {}", e);
            }
        }

        Commands::Debug { project, file } => {
            println!("--- [3/4] Debugger ---");

            let args = debugger::DebugArgs {
                project_name: project.clone(),
                file_name: file.clone(),
            };

            if let Err(e) = debugger::run(args) {
                eprintln!("❌ Debugger Error: {}", e);
            }
        }

        Commands::Build {
            project,
            resolutions,
            all,
        } => {
            println!("--- [4/4] Builder ---");

            let args = builder::BuildArgs {
                project_name: project.clone(),
                resolutions: resolutions.clone(),
                build_all: *all,
            };

            if let Err(e) = builder::run(args) {
                eprintln!("❌ Build Error: {}", e);
            }
        }

        Commands::Run { target } => {
            println!("--- [5/5] Watchdog Runner ---");
            let args = runner::RunArgs {
                target: target.clone(),
            };
            if let Err(e) = runner::run(args) {
                eprintln!("❌ Runner Error: {}", e);
            }
        }
    }
}
