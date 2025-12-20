import json
import subprocess
import sys
from pathlib import Path


def check_dependencies():
    try:
        subprocess.run(["yt-dlp", "--version"], capture_output=True, check=True)
        subprocess.run(["ffmpeg", "-version"], capture_output=True, check=True)
        subprocess.run(["ffprobe", "-version"], capture_output=True, check=True)
    except Exception:
        print("Error: yt-dlp, ffmpeg, or ffprobe not found in PATH.")
        sys.exit(1)


def get_video_info(file_path):
    cmd = [
        "ffprobe",
        "-v",
        "quiet",
        "-print_format",
        "json",
        "-show_streams",
        str(file_path),
    ]
    result = subprocess.run(cmd, capture_output=True, text=True)
    data = json.loads(result.stdout)
    for stream in data.get("streams", []):
        if stream.get("codec_type") == "video":
            w = int(stream.get("width"))
            h = int(stream.get("height"))
            fps_eval = eval(stream.get("avg_frame_rate"))
            return w, h, round(fps_eval)
    return 0, 0, 0


def download_and_prep():
    url = input("Enter YouTube URL: ").strip()
    if not url:
        return

    print("\n--- Target Resolution ---")
    print("[1] 720p")
    print("[2] 1080p")
    print("[3] 1440p")
    print("[4] 2160p")

    res_choice = input("Select choice [1-4]: ").strip()

    res_map = {
        "1": (1280, 720),
        "2": (1920, 1080),
        "3": (2560, 1440),
        "4": (3840, 2160),
    }

    tw, th = res_map.get(res_choice, (1920, 1080))

    print("\n--- FPS Strategy ---")
    print("Higher FPS requires more storage/RAM for the bin file.")
    print("[1] 30 FPS")
    print("[2] 60 FPS")
    print("[3] 120 FPS")
    print("[4] 144 FPS")
    print("[5] 165 FPS")

    fps_choice = input("Select choice [1-5]: ").strip()

    fps_map = {"1": 30, "2": 60, "3": 120, "4": 144, "5": 165}

    tfps = fps_map.get(fps_choice, 30)

    project_name = input("\nEnter Project Name: ").strip()
    if not project_name:
        project_name = "default_project"

    output_dir = Path(__file__).resolve().parent.parent / "assets" / project_name
    output_dir.mkdir(parents=True, exist_ok=True)

    temp_raw = output_dir / "temp_raw.mp4"
    final_video = output_dir / f"{project_name}.mp4"
    final_audio = output_dir / f"{project_name}.ogg"

    print("\n[1/3] Downloading source...")
    subprocess.run(
        [
            "yt-dlp",
            "-f",
            "bestvideo+bestaudio/best",
            url,
            "-o",
            str(temp_raw),
            "--merge-output-format",
            "mp4",
        ]
    )

    cw, ch, cfps = get_video_info(temp_raw)
    print(f"Source: {cw}x{ch} @ {cfps} FPS")

    filters = []

    if cw != tw or ch != th:
        filters.append(
            f"scale={tw}:{th}:force_original_aspect_ratio=increase:flags=lanczos"
        )
        filters.append(f"crop={tw}:{th}")
        filters.append("setsar=1")

    if cfps != tfps:
        filters.append(f"fps={tfps}")

    filters.append("format=gray")
    filters.append("unsharp=5:5:1.0:5:5:0.0")
    filters.append("eq=contrast=1000:saturation=0")

    encode_cmd = ["ffmpeg", "-i", str(temp_raw)]

    if filters:
        encode_cmd += ["-vf", ",".join(filters)]

    encode_cmd += [
        "-c:v",
        "h264_nvenc",
        "-preset",
        "p7",
        "-rc",
        "constqp",
        "-qp",
        "10",
        "-an",
        "-y",
        str(final_video),
    ]

    print(f"\n[2/3] Processing Video ({tw}x{th} @ {tfps} FPS)...")
    subprocess.run(encode_cmd)

    print("\n[3/3] Extracting Audio...")
    subprocess.run(
        [
            "ffmpeg",
            "-i",
            str(temp_raw),
            "-vn",
            "-acodec",
            "libvorbis",
            "-q:a",
            "5",
            "-y",
            str(final_audio),
        ]
    )

    if temp_raw.exists():
        temp_raw.unlink()
    print(f"\n--- SUCCESS: {project_name} Ready ---")


if __name__ == "__main__":
    check_dependencies()
    download_and_prep()
