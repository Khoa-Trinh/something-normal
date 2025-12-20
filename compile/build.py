import argparse
import os
import subprocess
from pathlib import Path

CARGO_PKG_NAME = "pixel_shell"
RESOLUTIONS = ["720p", "1080p", "1440p", "2160p"]


def get_available_builds():
    root = Path(__file__).resolve().parent.parent
    assets_dir = root / "assets"

    available = []
    if not assets_dir.exists():
        print(f"Error: Assets folder not found at {assets_dir}")
        return []

    for project_dir in sorted(assets_dir.iterdir()):
        if project_dir.is_dir():
            project_name = project_dir.name
            audio_path = project_dir / f"{project_name}.ogg"

            if not audio_path.exists():
                continue

            for res in RESOLUTIONS:
                bin_path = project_dir / f"{project_name}_{res}.bin"
                if bin_path.exists():
                    available.append(
                        {
                            "project": project_name,
                            "res": res,
                            "bin": bin_path,
                            "audio": audio_path,
                        }
                    )
    return available


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--select", type=str, help="Selection string (e.g. 'all', '1,3')"
    )
    args = parser.parse_args()

    root = Path(__file__).resolve().parent.parent
    dist_dir = root / "dist"
    dist_dir.mkdir(exist_ok=True)

    builds = get_available_builds()

    if not builds:
        print("No valid assets found. Run video-to-bin.py first!")
        return

    if args.select:
        selection = args.select.lower().strip()
    else:
        print("\n--- Available Build Targets ---")
        for idx, item in enumerate(builds):
            print(f"  [{idx + 1}] {item['project']} @ {item['res']}")
        selection = input("\nSelect targets (e.g. '1, 3' or 'all'): ").strip().lower()

    to_build = []
    if selection == "all":
        to_build = builds
    else:
        try:
            indices = [int(i.strip()) - 1 for i in selection.split(",") if i.strip()]
            to_build = [builds[i] for i in indices if 0 <= i < len(builds)]
        except (ValueError, IndexError):
            print("Invalid selection.")
            return

    if not to_build:
        print("Nothing selected.")
        return

    for item in to_build:
        project = item["project"]
        res = item["res"]
        final_exe_name = f"{project}_{res}.exe"

        print(f"\n🚀 [Building] {final_exe_name}...")

        env = os.environ.copy()
        env["BIN_PATH"] = str(item["bin"].absolute())
        env["AUDIO_PATH"] = str(item["audio"].absolute())

        try:
            subprocess.run(
                ["cargo", "build", "--release", "--features", f"res_{res}"],
                env=env,
                check=True,
                cwd=str(root),
            )

            generated_exe = root / "target" / "release" / f"{CARGO_PKG_NAME}.exe"

            if generated_exe.exists():
                target_path = dist_dir / final_exe_name
                if target_path.exists():
                    target_path.unlink()

                generated_exe.rename(target_path)
                print(f"✅ Success: {target_path}")
            else:
                print(f"❌ Error: Compiled binary not found at {generated_exe}")

        except subprocess.CalledProcessError:
            print(f"❌ Build Failed for {final_exe_name}")

    print(f"\n--- Process Complete! Files are in: {dist_dir} ---")


if __name__ == "__main__":
    main()
