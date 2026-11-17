"""读图工具：把 PNG 转成 BMP 后逐像素读，避免引入 Pillow 依赖。

用法（供 parity.py 调用）：
    from image import load
    img = load("shot.png")
    img.size()            # (w, h)
    img.pixel(x, y)       # (r, g, b)
"""

import os
import struct
import subprocess
import tempfile


class Image:
    def __init__(self, width, height, pixels):
        self.width = width
        self.height = height
        self._pixels = pixels

    def size(self):
        return (self.width, self.height)

    def pixel(self, x, y):
        return self._pixels[y * self.width + x]

    def bbox_of_color(self, target, tolerance=6, region=None, min_run=40):
        """返回与 target 相近像素的外接矩形；只统计足够长的水平连续段，
        避免被抗锯齿的零散像素带偏。"""
        x1, y1, x2, y2 = region or (0, 0, self.width, self.height)
        left, top, right, bottom = self.width, self.height, -1, -1
        for y in range(max(0, y1), min(y2, self.height)):
            row = y * self.width
            run_start = None
            for x in range(max(0, x1), min(x2, self.width)):
                r, g, b = self._pixels[row + x]
                match = (
                    abs(r - target[0]) <= tolerance
                    and abs(g - target[1]) <= tolerance
                    and abs(b - target[2]) <= tolerance
                )
                if match and run_start is None:
                    run_start = x
                elif not match and run_start is not None:
                    if x - run_start >= min_run:
                        left = min(left, run_start)
                        right = max(right, x - 1)
                        top = min(top, y)
                        bottom = max(bottom, y)
                    run_start = None
            if run_start is not None and x2 - run_start >= min_run:
                left = min(left, run_start)
                right = max(right, x2 - 1)
                top = min(top, y)
                bottom = max(bottom, y)
        if right < 0:
            return None
        return (left, top, right, bottom)

    def first_band_of_color(self, target, tolerance=2, min_run=40, max_gap=6):
        """从上往下找第一块「连续若干行都有同色长横条」的区域，用于定位输入框之类
        靠底色识别的元素（避免把下方同色的其它元素也算进来）。"""
        band = None
        last_row = None
        for y in range(self.height):
            row = y * self.width
            run_best = None
            run_start = None
            for x in range(self.width):
                r, g, b = self._pixels[row + x]
                match = (
                    abs(r - target[0]) <= tolerance
                    and abs(g - target[1]) <= tolerance
                    and abs(b - target[2]) <= tolerance
                )
                if match and run_start is None:
                    run_start = x
                elif not match and run_start is not None:
                    if x - run_start >= min_run and (run_best is None or x - run_start > run_best[1] - run_best[0]):
                        run_best = (run_start, x - 1)
                    run_start = None
            if run_start is not None and self.width - run_start >= min_run:
                if run_best is None or self.width - run_start > run_best[1] - run_best[0]:
                    run_best = (run_start, self.width - 1)

            if run_best is not None:
                if band is not None and last_row is not None and y - last_row <= max_gap:
                    band = (
                        min(band[0], run_best[0]),
                        band[1],
                        max(band[2], run_best[1]),
                        y,
                    )
                elif band is None:
                    band = (run_best[0], y, run_best[1], y)
                else:
                    return band
                last_row = y
            elif band is not None and last_row is not None and y - last_row > max_gap:
                return band
        return band

    def count_near(self, target, tolerance=6):
        """全图里与目标颜色相近的像素数，用来判断页面是否被遮挡/压暗。"""
        count = 0
        for r, g, b in self._pixels:
            if (
                abs(r - target[0]) <= tolerance
                and abs(g - target[1]) <= tolerance
                and abs(b - target[2]) <= tolerance
            ):
                count += 1
        return count

    def dominant_text_color(self, region, max_luminance=220):
        """区域内出现最多的「文字色」：只统计明显暗于底色的像素，
        取众数，避免被光标/分割线这类少量像素带偏。"""
        x1, y1, x2, y2 = region
        counts = {}
        for y in range(max(0, y1), min(y2, self.height)):
            row = y * self.width
            for x in range(max(0, x1), min(x2, self.width)):
                r, g, b = self._pixels[row + x]
                if (r + g + b) / 3 > max_luminance:
                    continue
                key = (r, g, b)
                counts[key] = counts.get(key, 0) + 1
        if not counts:
            return None
        return max(counts.items(), key=lambda item: item[1])[0]

    def darkest(self, region):
        """区域内最暗的像素，约等于文字笔画颜色。"""
        x1, y1, x2, y2 = region
        best = None
        best_sum = 1 << 30
        for y in range(max(0, y1), min(y2, self.height)):
            row = y * self.width
            for x in range(max(0, x1), min(x2, self.width)):
                r, g, b = self._pixels[row + x]
                if r + g + b < best_sum:
                    best_sum = r + g + b
                    best = (r, g, b)
        return best


def load(path):
    """用 sips 把 PNG 转成 BMP，再按 BMP 头部解析像素。"""
    with tempfile.NamedTemporaryFile(suffix=".bmp", delete=False) as handle:
        bmp_path = handle.name
    try:
        subprocess.run(
            ["sips", "-s", "format", "bmp", path, "--out", bmp_path],
            check=True,
            capture_output=True,
        )
        with open(bmp_path, "rb") as handle:
            data = handle.read()
    finally:
        if os.path.exists(bmp_path):
            os.unlink(bmp_path)

    if data[:2] != b"BM":
        raise ValueError(f"不是 BMP 数据: {path}")
    offset = struct.unpack_from("<I", data, 10)[0]
    width = struct.unpack_from("<i", data, 18)[0]
    height = struct.unpack_from("<i", data, 22)[0]
    bit_count = struct.unpack_from("<H", data, 28)[0]
    if bit_count not in (24, 32):
        raise ValueError(f"不支持 {bit_count} 位 BMP")

    top_down = height < 0
    height = abs(height)
    bytes_per_pixel = bit_count // 8
    stride = ((width * bytes_per_pixel) + 3) & ~3

    pixels = [(0, 0, 0)] * (width * height)
    for row in range(height):
        source_row = row if top_down else height - 1 - row
        base = offset + source_row * stride
        for column in range(width):
            position = base + column * bytes_per_pixel
            b, g, r = data[position], data[position + 1], data[position + 2]
            pixels[row * width + column] = (r, g, b)
    return Image(width, height, pixels)
