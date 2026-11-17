#!/usr/bin/env python3
"""Render the app icon from the SVG sources into every format the app and packages use.

Needs Inkscape (SVG -> PNG) and Pillow (ICO, ICNS). Run from anywhere:
    python3 packaging/icon/make_icons.py
"""
import subprocess
import tempfile
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
ASSETS = ROOT / "app" / "assets"

# Which artwork draws which size: small sizes get simplified versions.
SOURCES = {16: "icon-16.svg", 24: "icon-16.svg", 32: "icon-32.svg", 48: "icon-32.svg"}
SIZES = [16, 24, 32, 48, 64, 128, 256, 512, 1024]


def render(svg: Path, size: int, out: Path) -> None:
    subprocess.run(
        ["inkscape", str(svg), "--export-type=png", f"--export-filename={out}", "-w", str(size), "-h", str(size)],
        check=True,
        capture_output=True,
    )


def main() -> None:
    ASSETS.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp:
        imgs = {}
        for size in SIZES:
            png = Path(tmp) / f"{size}.png"
            render(HERE / SOURCES.get(size, "icon.svg"), size, png)
            imgs[size] = Image.open(png).convert("RGBA")
            imgs[size].load()

    # Window icon at runtime (miniquad wants 16, 32 and 64 px).
    for size in (16, 32, 64):
        imgs[size].save(ASSETS / f"icon-{size}.png")
    # Windows: embedded in the .exe by app/build.rs.
    ico = [imgs[s] for s in (16, 24, 32, 48, 64, 128, 256)]
    ico[-1].save(ASSETS / "icon.ico", sizes=[(i.width, i.height) for i in ico], append_images=ico[:-1])
    # macOS bundle icon.
    icns = [imgs[s] for s in (16, 32, 64, 128, 256, 512, 1024)]
    icns[-1].save(HERE / "AppIcon.icns", append_images=icns[:-1])
    # Linux desktop icon and README.
    imgs[256].save(HERE / "icon-256.png")
    print("icons written to", ASSETS, "and", HERE)


if __name__ == "__main__":
    main()
