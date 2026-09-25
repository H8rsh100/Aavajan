# Contributing to Aavajan

Aavajan is intentionally small: a Rust terminal renderer, a committed particle mask, and a clear animation loop. Please keep changes focused and preserve the lightweight runtime.

## Before you push

Run the same checks used by CI:

```text
cargo fmt --check
cargo check --locked
cargo test --locked
cargo clippy --locked -- -D warnings
```

For a visual change, also run a bounded preview:

```text
cargo run --release -- --plain
cargo run --release -- --frames 120
```

The first command prints a formed snapshot without ANSI control codes. The second command exercises the formation phase and exits automatically.

## Artwork changes

The source image is `Screenshot 2026-09-25 140449.png`. If the artwork changes, regenerate the compact mask with:

```text
python -m pip install pillow
python tools/extract_mask.py
```

Review the resulting `src/shape_mask.txt`, run the Rust checks, and commit the source image and generated mask together. The runtime should not gain an image-decoding dependency.

## Commit style

Make each commit a working, verifiable milestone. Keep the README and CLI help accurate when behavior changes. Do not commit generated GIFs, videos, or terminal casts; the recording tape belongs in source control, while the output stays local.
