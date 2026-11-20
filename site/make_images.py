#!/usr/bin/env python3
"""Render the website's pictures with the app itself (needs a desktop session and Pillow).

    python3 site/make_images.py

hero.jpg   the sample picture in three bands: original, value levels, colour levels
phone.png  the phone layout (Picture pane) with a clicked spot and its recipe
desktop.png  the desktop window, same as app/shots/now.png
"""
import subprocess
import tempfile
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
APP = HERE.parent / "app"
IMG = HERE / "img"
# Canvas background behind the picture (painting_view.rs: Color::new(0.08, 0.085, 0.095)).
CANVAS = (20, 22, 24)


def shot(out: Path, *args: str) -> Image.Image:
    subprocess.run(["cargo", "run", "-q", "--release", "--", "--screenshot", str(out), *args], cwd=APP, check=True)
    return Image.open(out).convert("RGB")


def picture_only(img: Image.Image) -> Image.Image:
    """The picture drawn on the canvas, cut out of a screenshot of the big window."""
    w, h = img.size
    px = img.load()
    # the canvas lies between the side panels; scan its middle row and column
    xs = [x for x in range(w) if max(abs(a - b) for a, b in zip(px[x, h // 2], CANVAS)) > 12]
    ys = [y for y in range(h) if max(abs(a - b) for a, b in zip(px[w // 2, y], CANVAS)) > 12]
    cx = [x for x in xs if 300 < x < w - 480]
    cy = [y for y in ys if 60 < y < h - 70]
    return img.crop((min(cx), min(cy), max(cx) + 1, max(cy) + 1))


def main() -> None:
    IMG.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp:
        views = [picture_only(shot(Path(tmp) / f"{v}.png", "--view", v, "--window", "2600x1500")) for v in ("original", "values", "colours")]
        w, h = views[0].size
        hero = Image.new("RGB", (w, h))
        for k, v in enumerate(views):
            box = (k * w // 3, 0, (k + 1) * w // 3, h)
            hero.paste(v.resize((w, h)).crop(box), box[:2])
        hero.thumbnail((1800, 1800), Image.LANCZOS)
        hero.save(IMG / "hero.jpg", quality=86, optimize=True, progressive=True)
        shot(Path(tmp) / "phone.png", "--window", "390x780").save(IMG / "phone.png", optimize=True)
        shot(Path(tmp) / "desktop.png").save(IMG / "desktop.png", optimize=True)
    print("pictures written to", IMG)


if __name__ == "__main__":
    main()
