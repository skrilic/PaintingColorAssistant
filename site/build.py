#!/usr/bin/env python3
"""Builds the project website (GitHub Pages) into a folder, with the web app under app/.

    python3 site/build.py [--out _site] [--app app/target/web]

Pages: Home (home.html), Changelog (CHANGELOG.md) and Privacy policy (PRIVACY.md), all in
layout.html. The download buttons point at the files of the latest GitHub release, read
with the `gh` command line tool; without it they point at the releases page.
Only the Python standard library is used.
"""
import argparse
import datetime
import html
import json
import re
import shutil
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
REPO = "skrilic/PaintingColorAssistant"
RELEASES = f"https://github.com/{REPO}/releases"
# Release file name endings (see .github/workflows/release.yml) for each download button.
PLATFORMS = {"mac": "-macos-universal.zip", "windows": "-windows-x86_64.zip", "linux": "-linux-x86_64.tar.gz"}


def latest_release() -> dict | None:
    try:
        out = subprocess.run(
            ["gh", "release", "view", "--repo", REPO, "--json", "tagName,publishedAt,assets,url"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
        return json.loads(out)
    except (OSError, subprocess.CalledProcessError, json.JSONDecodeError):
        return None


def day(iso: str) -> str:
    d = datetime.date.fromisoformat(iso[:10])
    return f"{d.day} {d.strftime('%B')} {d.year}"


def download_fields(release: dict | None) -> dict:
    fields = {"release_page": RELEASES}
    for key, ending in PLATFORMS.items():
        asset = next((a for a in (release or {}).get("assets", []) if a["name"].endswith(ending)), None)
        fields[f"{key}_url"] = asset["url"] if asset else RELEASES
        fields[f"{key}_size"] = f"{asset['size'] / 1e6:.1f} MB" if asset else "on the releases page"
    if release:
        fields["release_page"] = release["url"]
        version = release["tagName"].lstrip("v")
        fields["release_line"] = f"Version {version}, released {day(release['publishedAt'])}."
    else:
        fields["release_line"] = "All versions are on the releases page."
    return fields


def inline(text: str) -> str:
    """Markdown inline marks used in CHANGELOG.md and PRIVACY.md: links, bold, italic, code."""
    text = html.escape(text, quote=False)
    text = re.sub(r"`([^`]+)`", r"<code>\1</code>", text)
    text = re.sub(r"\[([^\]]+)\]\(([^)\s]+)\)", r'<a href="\2">\1</a>', text)
    text = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", text)
    text = re.sub(r"(?<![*\w])\*([^*]+)\*(?!\w)", r"<em>\1</em>", text)
    return text.replace(" – ", " &ndash; ")


def markdown(md: str) -> str:
    """The small part of Markdown these documents use: headings, paragraphs, - lists."""
    out, para, items = [], [], []

    def flush():
        if para:
            out.append(f"<p>{inline(' '.join(para))}</p>")
            para.clear()
        if items:
            out.append("<ul>" + "".join(f"<li>{inline(i)}</li>" for i in items) + "</ul>")
            items.clear()

    for line in md.splitlines():
        stripped = line.strip()
        heading = re.match(r"(#{1,3}) (.*)", stripped)
        if not stripped:
            flush()
        elif heading:
            flush()
            n = len(heading.group(1))
            out.append(f"<h{n}>{inline(heading.group(2))}</h{n}>")
        elif stripped.startswith("- "):
            if para:
                flush()
            items.append(stripped[2:])
        elif items and line.startswith("  "):
            items[-1] += " " + stripped
        else:
            para.append(stripped)
    flush()
    return "\n".join(out)


def fill(template: str, fields: dict) -> str:
    return re.sub(r"\{\{(\w+)\}\}", lambda m: str(fields.get(m.group(1), "")), template)


def page(name: str, title: str, content: str) -> str:
    fields = {"title": title, "page": name, "content": content}
    for tab in ("home", "changelog", "privacy"):
        fields[f"current_{tab}"] = ' aria-current="page"' if tab == name else ""
    return fill((HERE / "layout.html").read_text(), fields)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=str(ROOT / "_site"))
    ap.add_argument("--app", help="built web app folder (app/web/build.sh output), copied to app/")
    args = ap.parse_args()
    out = Path(args.out)
    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True)

    release = latest_release()
    home = fill((HERE / "home.html").read_text(), download_fields(release))
    (out / "index.html").write_text(page("home", "Painting Colour Assistant – a painting plan from a reference picture", home))
    for name, source, title in (("changelog", "CHANGELOG.md", "Changelog"), ("privacy", "PRIVACY.md", "Privacy policy")):
        body = f'        <article class="prose">\n{markdown((ROOT / source).read_text())}\n        </article>'
        (out / f"{name}.html").write_text(page(name, f"{title} – Painting Colour Assistant", body))

    shutil.copy(HERE / "style.css", out)
    shutil.copy(HERE / "sw.js", out)
    shutil.copytree(HERE / "img", out / "img")
    icons = out / "icons"
    icons.mkdir()
    for icon in ("favicon-32.png", "apple-touch-icon.png", "icon-192.png"):
        shutil.copy(ROOT / "app" / "web" / "icons" / icon, icons)
    (out / ".nojekyll").write_text("")
    if args.app:
        shutil.copytree(args.app, out / "app")
    print(f"website in {out}" + ("" if release else " (no release info: downloads link to the releases page)"))


if __name__ == "__main__":
    main()
