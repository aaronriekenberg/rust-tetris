pub mod game;
pub mod tetromino;

#[cfg(feature = "tui")]
pub mod render;

// ── WebAssembly bindings ──────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// A thin WebAssembly wrapper around `GameState`.
///
/// Build with:
/// ```
/// wasm-pack build --target web -- --no-default-features
/// ```
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmGame {
    state: game::GameState,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmGame {
    /// Create a new game.
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmGame {
        WasmGame {
            state: game::GameState::new(),
        }
    }

    /// Advance one gravity tick.  Returns `false` when the game is over.
    pub fn tick(&mut self) -> bool {
        self.state.tick()
    }

    pub fn move_left(&mut self) {
        self.state.move_left();
    }

    pub fn move_right(&mut self) {
        self.state.move_right();
    }

    pub fn rotate(&mut self) {
        self.state.rotate();
    }

    pub fn soft_drop(&mut self) {
        self.state.soft_drop();
    }

    pub fn hard_drop(&mut self) {
        self.state.hard_drop();
    }

    pub fn toggle_pause(&mut self) {
        self.state.toggle_pause();
    }

    /// Return the board as a flat `Uint8Array` (row-major, 10×20 = 200 bytes).
    /// Value 0 = empty, 1 = ghost, otherwise an ANSI-256 colour index for the
    /// locked/active cell.
    pub fn board_flat(&self) -> Vec<u8> {
        self.state.board_flat()
    }

    pub fn score(&self) -> u32 {
        self.state.score
    }

    pub fn level(&self) -> u32 {
        self.state.level
    }

    pub fn lines_cleared(&self) -> u32 {
        self.state.lines_cleared
    }

    pub fn is_game_over(&self) -> bool {
        self.state.game_over
    }

    pub fn is_paused(&self) -> bool {
        self.state.paused
    }

    /// Milliseconds between gravity ticks at the current level.
    pub fn drop_interval_ms(&self) -> u32 {
        self.state.drop_interval_ms() as u32
    }
}
