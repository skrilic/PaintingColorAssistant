#!/usr/bin/env python3
"""Render the app icon from the SVG sources into every format the app and packages use.

Needs Inkscape or rsvg-convert (SVG -> PNG) and Pillow (ICO, ICNS). Run from anywhere:
    python3 packaging/icon/make_icons.py             # everything
    python3 packaging/icon/make_icons.py --web-only  # only the web app icons (app/web/icons)
"""
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
ASSETS = ROOT / "app" / "assets"
WEB_ICONS = ROOT / "app" / "web" / "icons"
# Full-bleed background for icons the system crops to its own shape (Android "maskable",
# iOS home screen): a mid tone of the icon's own background gradient.
BLEED = (32, 34, 40, 255)

# Which artwork draws which size: small sizes get simplified versions.
SOURCES = {16: "icon-16.svg", 24: "icon-16.svg", 32: "icon-32.svg", 48: "icon-32.svg"}
SIZES = [16, 24, 32, 48, 64, 128, 256, 512, 1024]
WEB_SIZES = [192, 512, 1024]


def render(svg: Path, size: int, out: Path) -> None:
    if shutil.which("inkscape"):
        cmd = ["inkscape", str(svg), "--export-type=png", f"--export-filename={out}", "-w", str(size), "-h", str(size)]
    else:
        cmd = ["rsvg-convert", "-w", str(size), "-h", str(size), "-o", str(out), str(svg)]
    subprocess.run(cmd, check=True, capture_output=True)


def full_bleed(img: Image.Image, size: int) -> Image.Image:
    """The icon on an opaque square; its artwork stays inside the central safe circle."""
    out = Image.new("RGBA", img.size, BLEED)
    out.alpha_composite(img)
    return out.convert("RGB").resize((size, size), Image.LANCZOS)


def web_icons(imgs: dict) -> None:
    """Web app (PWA) icons, listed in app/web/manifest.webmanifest and index.html."""
    WEB_ICONS.mkdir(parents=True, exist_ok=True)
    imgs[192].save(WEB_ICONS / "icon-192.png")
    imgs[512].save(WEB_ICONS / "icon-512.png")
    full_bleed(imgs[1024], 512).save(WEB_ICONS / "icon-maskable-512.png")
    full_bleed(imgs[1024], 180).save(WEB_ICONS / "apple-touch-icon.png")
    imgs[32].save(WEB_ICONS / "favicon-32.png")


def main() -> None:
    web_only = "--web-only" in sys.argv[1:]
    sizes = [32] + WEB_SIZES if web_only else sorted(set(SIZES + WEB_SIZES))
    ASSETS.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp:
        imgs = {}
        for size in sizes:
            png = Path(tmp) / f"{size}.png"
            render(HERE / SOURCES.get(size, "icon.svg"), size, png)
            imgs[size] = Image.open(png).convert("RGBA")
            imgs[size].load()

    web_icons(imgs)
    if web_only:
        print("web icons written to", WEB_ICONS)
        return
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
