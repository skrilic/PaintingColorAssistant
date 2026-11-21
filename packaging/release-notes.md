## Web app (phones, tablets, any browser)

Open **https://skrilic.github.io/PaletteCraft/app/** – no download needed. To install it:
on iPhone and iPad tap *Share → Add to Home Screen* in Safari; on Android tap
*Install app* (or *⋮ → Add to Home screen*) in Chrome; in Chrome or Edge on a computer use
the install icon in the address bar. Once opened it also works without a network
connection. Palettes are kept in the browser, separately from the desktop app.

## Downloads

| System | File |
|---|---|
| Linux (x86_64, glibc 2.35+) | `palettecraft-*-linux-x86_64.tar.gz` |
| macOS 11+ (Apple Silicon and Intel) | `palettecraft-*-macos-universal.zip` |
| Windows 10/11 (x86_64) | `palettecraft-*-windows-x86_64.zip` |

`SHA256SUMS.txt` lists the checksums.

## Installing

**Linux** – unpack and run `./palettecraft`, or run `./install.sh` to add it
to your applications menu with its icon (`./install.sh --uninstall` removes it). The *Open picture...* dialog uses
the XDG desktop portal (standard on GNOME and KDE). If no dialog appears, drag the picture
onto the window or pass its path on the command line.

**macOS** – unzip and move *PaletteCraft.app* to Applications. The app is not
notarised, so the first launch is blocked: open it once, then go to
*System Settings → Privacy & Security* and click *Open Anyway*. Or in Terminal:
`xattr -dr com.apple.quarantine "/Applications/PaletteCraft.app"`.

**Windows** – unzip and run *PaletteCraft.exe*. The app is not signed, so
SmartScreen may warn: click *More info → Run anyway*.

Palettes are saved in your user settings folder and survive updates (the web app keeps
its own, in the browser).

## Support

If the app helps your painting, you can leave a tip on
[Ko-fi](https://ko-fi.com/Q6X32AWI41) – thank you!
