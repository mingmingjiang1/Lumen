"""三端等价度检查的小工具集（不依赖第三方库）。

子命令：
    centered <uiautomator.xml> <text> [tolerance]
        检查 Android 视图树里某段文字是否在其父容器里垂直居中

    placeholder <png> <r> <g> <b> [tolerance]
        全图找输入框底色（默认浅灰）区域，取其中最暗像素当占位符颜色，
        和期望值比较

    html-has <html 文件> <子串>...
        断言静态 HTML 里包含若干结构/样式标记

退出码：0 通过，1 失败，2 跳过（环境不具备）
"""

import sys
import xml.etree.ElementTree as ET

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from image import load  # noqa: E402


def bounds_of(node):
    raw = node.get("bounds", "")
    if not raw:
        return None
    parts = raw.replace("][", ",").strip("[]").split(",")
    if len(parts) != 4:
        return None
    return tuple(int(value) for value in parts)


def find_text_with_parents(node, text, parents):
    if node.get("text") == text:
        return (node, parents)
    for child in list(node):
        found = find_text_with_parents(child, text, parents + [node])
        if found:
            return found
    return None


def check_centered(argv):
    xml_path, text = argv[0], argv[1]
    tolerance = int(argv[2]) if len(argv) > 2 else 2
    try:
        root = ET.parse(xml_path).getroot()
    except ET.ParseError:
        print("跳过：视图树文件为空或不是有效 XML（应用可能还没起来）")
        return 2
    found = find_text_with_parents(root, text, [])
    if not found:
        print(f"跳过：视图树里没有找到「{text}」")
        return 2
    node, parents = found
    text_box = bounds_of(node)
    container = next(
        (parent for parent in reversed(parents) if bounds_of(parent)), None
    )
    if not text_box or container is None:
        print("跳过：找不到文字或容器坐标")
        return 2
    container_box = bounds_of(container)
    top_gap = text_box[1] - container_box[1]
    bottom_gap = container_box[3] - text_box[3]
    delta = abs(top_gap - bottom_gap)
    print(
        f"「{text}」上下留白 top={top_gap} bottom={bottom_gap} 差={delta}px "
        f"(容器高 {container_box[3] - container_box[1]}px)"
    )
    return 0 if delta <= tolerance else 1


def check_placeholder(argv):
    png_path = argv[0]
    expected = tuple(int(value) for value in argv[1:4])
    tolerance = int(argv[4]) if len(argv) > 4 else 12
    image = load(png_path)
    if len(argv) >= 9:
        # 显式给输入框矩形（Android 可以从 uiautomator 导出的坐标拿到）
        x, y, w, h = (int(value) for value in argv[5:9])
        field = (x, y, x + w, y + h)
    else:
        # 输入框底色 rgb(242,245,250) 和页面底色 rgb(247,248,251) 很接近，容差要收紧
        field = image.first_band_of_color((242, 245, 250), tolerance=2)
        if field is None:
            print("跳过：截图里没找到输入框底色 rgb(242,245,250)")
            return 2
    width, height = image.size()
    if (field[2] - field[0]) * (field[3] - field[1]) > 0.5 * width * height:
        print("跳过：输入框区域过大，无法定位")
        return 2
    # 页面底色（247,248,251）几乎不存在时，多半是界面被系统弹窗压暗/遮挡
    page_pixels = image.count_near((247, 248, 251), tolerance=8)
    if page_pixels < 0.05 * width * height:
        print(
            "跳过：画面里几乎没有页面底色，界面可能被系统弹窗压暗或遮挡"
            "（关掉弹窗后重跑）"
        )
        return 2
    inner = (field[0] + 6, field[1] + 4, field[2] - 6, field[3] - 4)
    sampled = image.dominant_text_color(inner)
    if sampled is None:
        print("跳过：输入框区域里没有文字像素")
        return 2
    delta = max(abs(sampled[i] - expected[i]) for i in range(3))
    print(
        f"占位符颜色采样 rgb{sampled} 期望 rgb{expected} 最大偏差={delta} "
        f"(输入框 {field[2] - field[0]}x{field[3] - field[1]}px)"
    )
    return 0 if delta <= tolerance else 1


def check_is_dark(argv):
    """截图几乎全黑 → 模拟器屏幕在休眠。"""
    image = load(argv[0])
    width, height = image.size()
    black = image.count_near((0, 0, 0), tolerance=12)
    ratio = black / float(width * height)
    print(f"黑色像素占比 {ratio:.1%}")
    return 0 if ratio > 0.9 else 1


def check_html(argv):
    path, needles = argv[0], argv[1:]
    content = open(path, encoding="utf-8").read()
    missing = [needle for needle in needles if needle not in content]
    if missing:
        print(f"HTML 缺少：{missing}")
        return 1
    print(f"HTML 结构检查通过（{len(needles)} 项）")
    return 0


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    command, argv = sys.argv[1], sys.argv[2:]
    handlers = {
        "centered": check_centered,
        "placeholder": check_placeholder,
        "html-has": check_html,
        "is-dark": check_is_dark,
    }
    if command not in handlers:
        print(f"未知子命令: {command}")
        return 2
    return handlers[command](argv)


if __name__ == "__main__":
    sys.exit(main())
