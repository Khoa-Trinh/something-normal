import argparse
import struct
import sys
from pathlib import Path

import cv2
import helpers
from tqdm import tqdm

VIDEO_EXTENSIONS = {".mp4", ".mkv", ".avi", ".mov", ".webm", ".flv"}

RESOLUTIONS = [
    ("720p", 1280),
    ("1080p", 1920),
    ("1440p", 2560),
    ("2160p", 3840),
]


def get_project_paths():
    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    assets_dir = project_root / "assets"
    if not assets_dir.exists():
        print(f"Error: Assets folder not found at {assets_dir}")
        sys.exit(1)
    return assets_dir


def select_project(assets_dir, project_index=None):
    projects = sorted([d for d in assets_dir.iterdir() if d.is_dir()])

    if not projects:
        print("No project folders found!")
        sys.exit(1)

    if project_index is not None:
        idx = project_index - 1
        if 0 <= idx < len(projects):
            print(f"Selected Project via CLI: {projects[idx].name}")
            return projects[idx]
        else:
            print(
                f"Error: Project index {project_index} is out of range (1-{len(projects)})"
            )
            sys.exit(1)

    print("\n--- Projects ---")
    for idx, p in enumerate(projects):
        print(f"[{idx + 1}] {p.name}")
    while True:
        try:
            c = int(input("Select Project: ")) - 1
            if 0 <= c < len(projects):
                return projects[c]
        except ValueError:
            pass


def select_hardware(debug_enabled: bool, force_mode=None):
    if force_mode:
        if force_mode == "cpu":
            return helpers.CpuEngine("bw", debug_enabled)
        elif force_mode == "gpu":
            if helpers.HAS_GPU:
                return helpers.GpuEngine("bw", debug_enabled)
            else:
                print("Error: GPU requested but not available.")
                sys.exit(1)

    print("\n--- Hardware Acceleration ---")
    print("[1] CPU (Numba Optimized) - Stable, Fast")
    if helpers.HAS_GPU:
        print("[2] GPU (NVIDIA CUDA) - Very Fast for 4K")
    else:
        print("[2] GPU (Not Detected - Install 'cupy-cuda12x')")

    while True:
        c = input("Select Hardware: ").strip()
        if c == "1":
            return helpers.CpuEngine("bw", debug_enabled)
        if c == "2":
            if not helpers.HAS_GPU:
                print("Error: CuPy not installed. Run: pip install cupy-cuda12x")
                continue
            print("Initializing GPU Engine...")
            return helpers.GpuEngine("bw", debug_enabled)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--project-index", type=int, help="1-based index of the project folder"
    )
    parser.add_argument(
        "--resolutions",
        type=str,
        help="Comma separated list (e.g. '720p,1080p') or 'all'",
    )
    parser.add_argument(
        "--hardware", type=str, choices=["cpu", "gpu"], help="Force cpu or gpu"
    )
    parser.add_argument("--debug", action="store_true", help="Enable debug frames")
    args = parser.parse_args()

    assets_dir = get_project_paths()
    project_path = select_project(assets_dir, args.project_index)

    targets = []
    if args.resolutions:
        req_res = args.resolutions.lower().strip()
        if req_res == "all":
            targets = RESOLUTIONS
        else:
            wanted = [r.strip() for r in req_res.split(",")]
            for r_name in wanted:
                match = next((x for x in RESOLUTIONS if x[0] == r_name), None)
                if match:
                    targets.append(match)
    else:
        print("\n--- Resolutions ---")
        print("Example: '1, 2' or 'all'")
        for i, (label, w) in enumerate(RESOLUTIONS):
            print(f"[{i + 1}] {label} ({w}px)")
        choice = input("Select: ").strip().lower()
        if choice == "all":
            targets = RESOLUTIONS
        else:
            idxs = [int(x) - 1 for x in choice.split(",") if x.strip().isdigit()]
            targets = [RESOLUTIONS[i] for i in idxs if 0 <= i < len(RESOLUTIONS)]

    debug_enabled = args.debug
    if not args.project_index and not args.debug:
        d_choice = (
            input("Generate debug frames every 60 frames? (y/n): ").strip().lower()
        )
        debug_enabled = d_choice == "y"

    engine = select_hardware(debug_enabled, args.hardware)

    vid_path = next(
        (p for p in project_path.iterdir() if p.suffix in VIDEO_EXTENSIONS), None
    )
    if not vid_path:
        print("No video found!")
        sys.exit(1)

    cap_temp = cv2.VideoCapture(str(vid_path))
    detected_fps = int(cap_temp.get(cv2.CAP_PROP_FPS))
    cap_temp.release()
    if detected_fps == 0:
        detected_fps = 30
    print(f"\nDetected FPS: {detected_fps}")

    for res_name, target_w in targets:
        out_path = project_path / f"{project_path.name}_{res_name}.bin"
        print(f"\nProcessing {res_name}...")

        cap = cv2.VideoCapture(str(vid_path))
        total = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
        orig_w = cap.get(cv2.CAP_PROP_FRAME_WIDTH)
        orig_h = cap.get(cv2.CAP_PROP_FRAME_HEIGHT)
        target_h = int(target_w * (orig_h / orig_w))

        with open(out_path, "wb") as f:
            f.write(struct.pack("H", detected_fps))

            with tqdm(total=total) as pbar:
                while cap.isOpened():
                    ret, frame = cap.read()
                    if not ret:
                        break

                    rects = engine.process(frame, target_w, target_h)

                    for x, y, w, h in rects:
                        f.write(struct.pack("HHHH", int(x), int(y), int(w), int(h)))
                    f.write(struct.pack("HHHH", 0, 0, 0, 0))
                    pbar.update()
        cap.release()


if __name__ == "__main__":
    main()
