# Brief

**Question:** Build a desktop painting colour assistant: open an image, crop it, view a greyscale value study with N levels, a colour-reduced version with N levels, overlay a composition grid (rule of thirds, Fibonacci spiral, square grid), and for each reduced colour compute a Kubelka-Munk mixing recipe from a user-defined pigment palette (watercolor / acrylic / oil, with transparency per paint).

**Purpose:** Tool for an artist planning a painting; no original program to match, built from the request alone.

**Constraints:**
- Rust + macroquad in app/
- No target: build from request
- --screenshot and --self-test options required
- Only change files in app/

**Done when:** cargo build succeeds, screenshot shows the workspace with a sample image, self-test passes move/act/hud, palettes can be created and saved, recipes shown per colour.
