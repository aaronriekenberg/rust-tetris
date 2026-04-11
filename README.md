# rust-tetris

A TUI (Terminal User Interface) Tetris game written in Rust, with WebAssembly support.

## Features

- Classic Tetris gameplay on a 10×20 board
- All 7 standard tetrominoes (I, O, T, S, Z, J, L) in distinct colours
- Ghost piece showing where the current piece will land
- Wall-kick rotation system
- Soft drop and hard drop
- Line clearing with standard scoring (100 / 300 / 500 / 800 × level)
- Level progression (every 10 lines)
- Pause / resume
- WebAssembly-compatible game logic

## Controls

| Key | Action |
|-----|--------|
| ← / → | Move left / right |
| ↑ or Z | Rotate clockwise |
| ↓ | Soft drop (+1 point per row) |
| Space | Hard drop (+2 points per row) |
| P | Pause / resume |
| Q / Esc | Quit |

## Building & Running

```bash
# Run in debug mode
cargo run

# Run optimised release build
cargo run --release
```

## WebAssembly

The game logic is separated into a library crate (`rust_tetris`) that compiles to
WebAssembly.  Use [wasm-pack](https://rustwasm.github.io/wasm-pack/) to build:

```bash
wasm-pack build --target web -- --no-default-features
```

This produces a `pkg/` directory with the compiled `.wasm` file and JavaScript
bindings.  The `WasmGame` struct exposes:

| Method | Description |
|--------|-------------|
| `new()` | Create a new game |
| `tick()` | Advance gravity (returns `false` on game over) |
| `move_left()` / `move_right()` | Move piece |
| `rotate()` | Rotate piece |
| `soft_drop()` / `hard_drop()` | Drop piece |
| `toggle_pause()` | Pause / resume |
| `board_flat()` | 200-byte flat board state (`Uint8Array`) |
| `score()` / `level()` / `lines_cleared()` | Game stats |
| `is_game_over()` / `is_paused()` | Status flags |
| `drop_interval_ms()` | Current gravity interval in ms |
