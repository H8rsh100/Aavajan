# Aavajan Architecture

Aavajan is organized around one deliberately small pipeline:

```text
committed mask
      |
      v
particle targets -----> particle state -----> normalized framebuffer
                                             |             |
                                             v             v
                                      2x4 dot accumulation  ANSI output
                                             |             |
                                             +----> Braille glyphs
```

## Modules

### `src/shape.rs`

Reads `src/shape_mask.txt`, finds filled mask cells, and samples a bounded target set. The target coordinates are normalized to the 0..1 range so the same particle state works at any terminal size.

### `src/main.rs`

Owns the particle lifecycle:

- Aavahan: move from randomized arrival points toward the mask.
- Formation: ease each particle into its target position.
- Dharma: hold the silhouette with low-amplitude drift.
- Visarjan: move outward with a small tangential swirl and fade.

The CLI modes share the same renderer. `--plain` removes terminal control codes, `--frames` bounds a run, `--seed` makes a recording reproducible, and `--benchmark` measures the render path without drawing to the terminal.

### `src/braille.rs`

Maintains a floating-point light buffer at two horizontal by four vertical sub-pixels per terminal cell. Each sub-pixel receives particle energy and color. During output, ordered Bayer dithering converts accumulated light into a Unicode Braille dot pattern, while the weighted color becomes a 24-bit ANSI foreground.

The framebuffer can resize without changing normalized particle coordinates. The renderer clears and redraws the complete surface each frame, which keeps the animation deterministic and avoids stale trails.

## Artwork pipeline

The supplied PNG is converted once by `tools/extract_mask.py`. The generated mask is committed so normal Rust builds do not need Pillow, an image decoder, or the original file at runtime. The extractor is intentionally boring: threshold, downsample, and write text.

## Performance choices

- One runtime dependency: `crossterm` for terminal lifecycle and input.
- No runtime image parsing, allocation per particle, or scene graph.
- A bounded target set prevents terminal output from growing with source image resolution.
- Braille output is assembled into one contiguous buffer per frame and flushed once.
- `--benchmark` black-boxes a complete plain frame so the optimizer cannot discard the measured work.

## Extension points

A future live data source can feed the same particle state without changing the framebuffer. A network or security event stream could alter target energy, color, or dissolve timing while preserving the Aavahan-to-Visarjan narrative. A second palette can be introduced by changing the color assignment in `create_particles` and keeping the renderer unchanged.
