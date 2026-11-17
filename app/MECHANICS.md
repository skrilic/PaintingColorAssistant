# Painting Colour Assistant: mechanics sheet

There is no original program to match: everything below comes from the person's request
("request") or is a design choice made while building ("estimated"). The same values are
in `mechanics.json`.

## Flow and controls

| What | Value | Source |
|---|---|---|
| Screens | 1 Palettes, 2 Painting (tabs in the top bar) | request: "artist creates palettes at the beginning" |
| First start | opens on Palettes; later starts open on Painting | estimated |
| Open a picture | "Open picture..." file dialog, drop a file on the window, or pass a path on the command line | request |
| Pick a colour | left click on the picture (Original / Greyscale / Value views) gives a recipe for a 5x5 px average | estimated |
| Select a colour level | left click on the picture or a recipe row (Colour levels view) fades the other levels | estimated |
| Crop | "Crop..." then drag: outside the frame = new frame, inside = move, corner handle (12 px) = resize; "Apply crop", "Undo crop" | request |
| Crop shapes | Free, 1:1, 4:5, 3:4, 2:3, 5:4, 4:3, 3:2, 16:9 | estimated |
| Scroll lists | mouse wheel, 1 row per notch (library: 2 rows) | estimated |
| Text fields | click to type, Backspace deletes, Enter/Escape/click elsewhere finishes | estimated |
| Delete palette | needs a second click ("Click again") | estimated |
| Palettes saved | automatically after every change, JSON in the user settings folder (`PaintingColorAssistant/palettes.json`) | estimated |

## Picture analysis

| What | Value | Source |
|---|---|---|
| Working copy longest side | 1400 px (bigger pictures are scaled down for display and analysis; crops use the full original) | estimated |
| Views | Original, Greyscale, Value levels, Colour levels | request |
| Value levels | default 5, range 2 to 10 | request (number chosen by artist); default estimated |
| Value banding | L* (CIE lightness) between the 0.5% and 99.5% percentiles split in equal steps; shown as evenly spaced greys black to white | estimated |
| Greyscale | each pixel shown as neutral grey of the same L* | estimated |
| Colour levels | default 8, range 2 to 24 | request; default estimated |
| Colour reduction | k-means++ in CIE Lab, 60 000 sampled pixels, 12 iterations, fixed seed; levels sorted dark to light; shown colour = mean sRGB of its pixels | estimated |

## Kubelka-Munk mixing

Two-constant Kubelka-Munk per RGB channel in linear light. Each paint is described by
its masstone colour (full strength), its transparency and its tinting strength.

| What | Value | Source |
|---|---|---|
| Media | Watercolor, Acrylic, Oil | request |
| Scattering S, transparent | 0.12 | estimated |
| Scattering S, half-transparent | 0.45 | estimated |
| Scattering S, opaque | 1.6 | estimated |
| Watercolor model | paint layer of thickness = pigment load over paper; K solved so a full-strength layer shows the masstone | estimated |
| Paper colour | sRGB 247, 244, 236 | estimated |
| Water steps (pigment load) | 1.0, 0.75, 0.55, 0.40, 0.30, 0.22, 0.16, 0.11, 0.075, 0.05, 0.033, 0.02 | estimated |
| Water words | >=0.9 full strength, >=0.65 creamy, >=0.45 like milk, >=0.28 like coffee, >=0.15 like tea, >=0.07 light wash, else very pale wash | estimated (common painters' terms) |
| Acrylic/oil model | opaque body, R = R_inf(K/S); K = S x (1-R)^2 / 2R from the masstone | estimated |
| White for acrylic/oil | the palette's PW paint, otherwise Titanium White sRGB 246, 246, 242 opaque | estimated |
| White parts tried | 0, 1, 2, 3, 4, 6, 8, 12, 16, 24, 40 | estimated |
| Mixture | K and S added by parts x tinting strength | estimated |
| Tinting strength | 0.2 to 3.0, default 1.0 (library values per pigment) | estimated |
| Ratios tried | pairs 1..6 parts each, triples 1..4 parts each (reduced ratios only) | estimated |
| Paints per mix | default 3, choice 1 / 2 / 3 | estimated |
| Simplicity bonus | +0.8 dE per extra paint when ranking | estimated |
| Colour difference | CIE76 dE in Lab | estimated |
| Match words | <2.5 excellent, <5 good, <10 close, else rough | estimated |
| Recipe search budget | 12 ms per frame (rest continues next frame) | estimated |

## Palettes

| What | Value | Source |
|---|---|---|
| Paint fields | name, pigment code (e.g. PR206), transparency, masstone colour, tinting strength | request |
| Transparency choices | Transparent, Half-transparent, Opaque | request |
| Starter palettes | Watercolor 1, Watercolor 2, Acrylic 1, Acrylic 2, Oil 1 | request (examples) |
| Watercolor 1 | Crimson PR206 (T), Cobalt Blue PB28 (HT), Yellow Deep PY35 (O), Lemon Yellow PY175, Ultramarine PB29, Burnt Sienna PBr7, Phthalo Green PG7 | request for the first three; codes and the rest estimated |
| Pigment library | 45 common pigments with approximate masstone colours | estimated (not measured) |
| Mixing chart | every pair at 1:1 plus 5 lighter steps per paint (water 1, 0.55, 0.3, 0.16, 0.075 or white 0, 1, 3, 8, 24 parts) | estimated |

## Overlays

| What | Value | Source |
|---|---|---|
| Kinds | None, Rule of thirds, Fibonacci spiral, Grid | request |
| Rule of thirds | lines at 1/3 and 2/3, rings (7 px) at the four crossings | estimated |
| Fibonacci spiral | rectangles cut at 1/phi (phi = 1.618034) of the side, 12 steps, quarter-ellipse arcs, stretched to the picture; portrait pictures start with a horizontal cut; mirror sideways / up-down | estimated |
| Grid | square cells, default 6 on the long side, range 2 to 20, optional diagonals, column letters and row numbers | estimated |
| Line colour | White, Black, Red; strength 0.15 to 1.0, default 0.8 | estimated |

## Window

| What | Value | Source |
|---|---|---|
| Window | 1400 x 880, resizable | estimated |
| Left panel / right panel | 250 px / 430 px | estimated |
