# Aavajan - Ganesha Particle Renderer

Aavajan is a compact Rust terminal art experiment that turns a stylized Ganesha silhouette into a field of glowing Braille particles. The image is rendered with 24-bit ANSI color and a small 2x4 sub-pixel framebuffer, so the animation stays sharp even in a normal terminal.

The animation follows an 8.5-second loop:

1. **Aavahan and formation** - particles arrive and assemble for 4 seconds.
2. **Dharma** - the formed Ganesha holds with a gentle living shimmer for 2.5 seconds.
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
