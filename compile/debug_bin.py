import struct
from pathlib import Path

import cv2
import numpy as np


def debug_bin_file():
    script_dir = Path(__file__).resolve().parent
    assets_dir = script_dir.parent / "assets"

    print(f"Scanning: {assets_dir}")
    projects = [d for d in assets_dir.iterdir() if d.is_dir()]
    if not projects:
        print("No projects found!")
        return

    for i, p in enumerate(projects):
        print(f"[{i + 1}] {p.name}")
    choice = int(input("Select Project: ")) - 1
    project = projects[choice]

    bin_files = list(project.glob("*.bin"))
    for i, b in enumerate(bin_files):
        print(f"[{i + 1}] {b.name}")
    choice = int(input("Select Bin File: ")) - 1
    bin_path = bin_files[choice]

    print(f"\nAnalying: {bin_path.name}...")
    with open(bin_path, "rb") as f:
        fps_data = f.read(2)
        fps = struct.unpack("<H", fps_data)[0]
        print(f"Target FPS: {fps}")

        canvas_h, canvas_w = 1080, 1920
        cv2.namedWindow("Debug View", cv2.WINDOW_NORMAL)

        frame_idx = 0
        while True:
            canvas = np.zeros((canvas_h, canvas_w, 3), dtype=np.uint8)
            rect_count = 0

            while True:
                chunk = f.read(8)
                if not chunk:
                    break

                x, y, w, h = struct.unpack("<HHHH", chunk)

                if x == 0 and y == 0 and w == 0 and h == 0:
                    break

                rect_count += 1

                cv2.rectangle(canvas, (x, y), (x + w, y + h), (255, 255, 255), -1)

                cv2.rectangle(canvas, (x, y), (x + w, y + h), (0, 0, 255), 1)

            if not chunk:
                break

            cv2.putText(
                canvas,
                f"Frame: {frame_idx} | Rects: {rect_count}",
                (50, 50),
                cv2.FONT_HERSHEY_SIMPLEX,
                1,
                (0, 255, 0),
                2,
            )

            cv2.imshow("Debug View", canvas)
            key = cv2.waitKey(int(1000 / fps))
            if key == 27:
                break

            frame_idx += 1

    cv2.destroyAllWindows()


if __name__ == "__main__":
    debug_bin_file()
