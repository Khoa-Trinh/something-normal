from pathlib import Path

import cv2
import numpy as np
from numba import jit

try:
    import cupy as cp

    HAS_GPU = True
except ImportError:
    HAS_GPU = False
    cp = None

SCRIPT_DIR = Path(__file__).resolve().parent
DEBUG_DIR = SCRIPT_DIR / "debug_frames"
DEBUG_DIR.mkdir(exist_ok=True)


@jit(nopython=True, fastmath=True)
def extract_boxes_numba(mask):
    height, width = mask.shape
    boxes = []

    active_boxes = np.full(width, -1, dtype=np.int32)

    for y in range(height):
        x = 0
        while x < width:
            if mask[y, x] == 0:
                if active_boxes[x] != -1:
                    active_boxes[x] = -1
                x += 1
                continue

            start_x = x

            while x < width and mask[y, x] != 0:
                if x > start_x:
                    active_boxes[x] = -1
                x += 1

            w = x - start_x

            idx = active_boxes[start_x]
            merged = False

            if idx != -1:
                b_x = boxes[idx][0]
                b_w = boxes[idx][2]

                if b_x == start_x and b_w == w:
                    boxes[idx] = (b_x, boxes[idx][1], b_w, boxes[idx][3] + 1)
                    merged = True
                else:
                    active_boxes[start_x] = -1

            if not merged:
                new_idx = len(boxes)
                boxes.append((start_x, y, w, 1))
                active_boxes[start_x] = new_idx

    return boxes


def weld_gaps(mask):
    """
    Fixes the 'Horizontal Striping' issue.
    """

    v_kernel = np.ones((5, 1), np.uint8)
    mask = cv2.dilate(mask, v_kernel, iterations=1)

    fill_kernel = np.ones((3, 3), np.uint8)
    mask = cv2.morphologyEx(mask, cv2.MORPH_CLOSE, fill_kernel, iterations=2)

    return mask


class CpuEngine:
    def __init__(self, mode_str, debug_enabled=False):
        self.mode = mode_str
        self.debug_enabled = debug_enabled
        self.frame_count = 0
        print(f"Engine initialized: CPU (Numba) - Mode: {mode_str} (Bugfix: ON)")

    def process(self, frame, target_w, target_h):
        resized = cv2.resize(
            frame, (target_w, target_h), interpolation=cv2.INTER_NEAREST
        )

        if self.mode == "bw":
            gray = cv2.cvtColor(resized, cv2.COLOR_BGR2GRAY)

            _, mask = cv2.threshold(gray, 127, 255, cv2.THRESH_BINARY)
        else:
            hsv = cv2.cvtColor(resized, cv2.COLOR_BGR2HSV)
            lower = np.array([35, 50, 50])
            upper = np.array([85, 255, 255])
            mask_inv = cv2.inRange(hsv, lower, upper)
            mask = cv2.bitwise_not(mask_inv)

        mask = weld_gaps(mask)

        if self.debug_enabled and self.frame_count % 60 == 0:
            cv2.imwrite(str(DEBUG_DIR / f"frame_{self.frame_count}_cpu.png"), mask)

        self.frame_count += 1
        return extract_boxes_numba(mask)


class GpuEngine:
    def __init__(self, mode_str, debug_enabled=False):
        if not HAS_GPU:
            raise RuntimeError("CuPy not installed.")
        self.mode = mode_str
        self.debug_enabled = debug_enabled
        self.frame_count = 0
        print(f"Engine initialized: GPU (CUDA) - Mode: {mode_str} (Bugfix: ON)")

    def process(self, frame, target_w, target_h):
        resized = cv2.resize(
            frame, (target_w, target_h), interpolation=cv2.INTER_NEAREST
        )

        if cp is None:
            raise RuntimeError("CuPy unavailable")

        gpu_frame = cp.asarray(resized)

        if self.mode == "bw":
            b = gpu_frame[:, :, 0].astype(cp.float32)
            g = gpu_frame[:, :, 1].astype(cp.float32)
            r = gpu_frame[:, :, 2].astype(cp.float32)
            gray = 0.299 * r + 0.587 * g + 0.114 * b
            mask_gpu = (gray > 127).astype(cp.uint8) * 255
        else:
            b = gpu_frame[:, :, 0]
            g = gpu_frame[:, :, 1]
            r = gpu_frame[:, :, 2]
            is_green = (g > 90) & (g > r + 10) & (g > b + 10)
            mask_gpu = (~is_green).astype(cp.uint8) * 255

        mask_cpu = cp.asnumpy(mask_gpu)
        mask_cpu = weld_gaps(mask_cpu)

        if self.debug_enabled and self.frame_count % 60 == 0:
            cv2.imwrite(str(DEBUG_DIR / f"frame_{self.frame_count}_gpu.png"), mask_cpu)

        self.frame_count += 1
        return extract_boxes_numba(mask_cpu)
