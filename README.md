# Aavajan - Ganesha Particle Renderer

Aavajan is a compact Rust terminal art experiment that turns a stylized Ganesha silhouette into a field of glowing Braille particles. The image is rendered with 24-bit ANSI color and a small 2x4 sub-pixel framebuffer, so the animation stays sharp even in a normal terminal.

The animation follows a nine-second loop:

1. **Aavahan and formation** - particles arrive and assemble for 4 seconds.
2. **Dharma** - the formed Ganesha holds with a gentle living shimmer for 3 seconds.
3. **Visarjan** - particles sag downward, trail, cool toward blue, and dissolve like water over 2 seconds, then the next cycle begins immediately.

## The name

Aavajan blends **Aavahan** and **Visarjan**: arrival and departure. The same idea drives the renderer, where particles form a temporary image and then return to motion. It is also a small nod to the Ganpati festival cycle.

## Run it

Requirements:

- Rust stable toolchain
- A terminal with 24-bit ANSI color support, such as Windows Terminal, iTerm2, or a modern Linux terminal

```text
cargo run --release
```

Controls:

- `q` or `Esc` - exit
- `Ctrl+C` - exit

For a bounded recording or smoke test, render a fixed number of frames:

```text
cargo run --release -- --frames 540
```

At 60 frames per second, 540 frames gives one complete animation cycle. The renderer automatically caps the terminal dimensions to a comfortable range and defaults to an 80x24-sized presentation when the terminal reports its size.

For a plain formed snapshot without terminal control codes:

```text
cargo run --release -- --plain
```

For reproducible recordings, provide any unsigned 32-bit seed. The default seed is used when this option is omitted.

```text
cargo run --release -- --frames 540 --seed 1337
```

Use `cargo run --release -- --version` to print the package version.

To measure the renderer without writing terminal frames:

```text
cargo run --release -- --benchmark --frames 600
```

## How it works

- `src/braille.rs` accumulates colored light in a 2x4 sub-pixel grid, applies ordered dithering, and emits Unicode Braille plus ANSI truecolor escape sequences.
- `src/shape.rs` loads the committed `src/shape_mask.txt` and converts the filled artwork into bounded particle targets.
- `src/main.rs` handles the Aavahan-to-Visarjan animation, particle motion, terminal lifecycle, and bounded frame mode.
- The artwork is sampled once at build-preparation time. The running binary does not need an image decoder.

## Regenerate the artwork mask

The source artwork is `Ganesha.png`. The runtime uses the generated mask, while the optional Python helper makes the transformation reproducible:

```text
python -m pip install pillow
python tools/extract_mask.py
```

The extractor thresholds the source image, reduces it to a compact 60-column mask, and writes `src/shape_mask.txt`. Keep the generated mask committed so normal builds remain dependency-light.

## Verify

```text
cargo fmt --check
cargo check
cargo test
cargo clippy -- -D warnings
```

## Recording

A VHS tape is provided in `vhs/demo.tape`. It records a complete nine-second cycle at the compact terminal size. The generated GIF is ignored by Git so the repository stays focused on source and reproducible instructions.

For an asciinema cast on Windows, install asciinema and run:

```text
powershell -ExecutionPolicy Bypass -File scripts/record-asciinema.ps1
```

The helper accepts `-Frames`, `-Seed`, and `-Output` parameters and writes a local `.cast` file, which is also ignored by Git.
