from pathlib import Path
from PIL import Image

SOURCE = Path("Screenshot 2026-09-25 140449.png")
TARGET = Path("src/shape_mask.txt")
WIDTH = 60
HEIGHT = 84
THRESHOLD = 130
COVERAGE = 0.16


def main() -> None:
    image = Image.open(SOURCE).convert("L")
    pixels = image.load()
    scale_x = image.width / WIDTH
    scale_y = image.height / HEIGHT
    lines = []

    for row in range(HEIGHT):
        y0 = min(image.height - 1, int(row * scale_y))
        y1 = min(image.height, max(y0 + 1, int((row + 1) * scale_y)))
        line = []
        for col in range(WIDTH):
            x0 = min(image.width - 1, int(col * scale_x))
            x1 = min(image.width, max(x0 + 1, int((col + 1) * scale_x)))
            values = [
                pixels[x, y]
                for y in range(y0, y1)
                for x in range(x0, x1)
            ]
            coverage = sum(value < THRESHOLD for value in values) / len(values)
            line.append("#" if coverage > COVERAGE else " ")
        lines.append("".join(line).rstrip())

    while lines and not lines[-1]:
        lines.pop()
    TARGET.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {TARGET} with {len(lines)} rows")


if __name__ == "__main__":
    main()
