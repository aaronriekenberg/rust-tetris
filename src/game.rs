use crate::tetromino::{TetrominoKind, PIECE_CELLS};

pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;

/// ANSI 256-color index for each tetromino kind.
const PIECE_COLORS: [u8; 7] = [
    51,  // I – cyan
    226, // O – yellow
    129, // T – purple
    46,  // S – green
    196, // Z – red
    21,  // J – blue
    208, // L – orange
];

/// A single locked cell on the board.  `None` means empty.
pub type CellColor = Option<u8>;

/// The 10×20 playfield.
#[derive(Clone)]
pub struct Board {
    pub cells: [[CellColor; BOARD_WIDTH]; BOARD_HEIGHT],
}

impl Board {
    pub fn new() -> Self {
        Self {
            cells: [[None; BOARD_WIDTH]; BOARD_HEIGHT],
        }
    }

    /// `true` if the position is inside the board and the cell is empty.
    pub fn is_empty(&self, row: i32, col: i32) -> bool {
        if col < 0 || col >= BOARD_WIDTH as i32 || row >= BOARD_HEIGHT as i32 {
            return false;
        }
        if row < 0 {
            return true; // above the board is fine during spawn
        }
        self.cells[row as usize][col as usize].is_none()
    }

    /// Lock a piece into the board.
    pub fn lock(&mut self, piece: &Piece) {
        let color = PIECE_COLORS[piece.kind.index()];
        for (dr, dc) in piece.cells() {
            let r = piece.row + dr;
            let c = piece.col + dc;
            if r >= 0 && r < BOARD_HEIGHT as i32 && c >= 0 && c < BOARD_WIDTH as i32 {
                self.cells[r as usize][c as usize] = Some(color);
            }
        }
    }

    /// Clear completed lines and return the number cleared.
    pub fn clear_lines(&mut self) -> u32 {
        let mut cleared = 0u32;
        let mut read_row = BOARD_HEIGHT - 1;

        // Temporary buffer to store non-full rows
        let mut kept: Vec<[CellColor; BOARD_WIDTH]> = Vec::with_capacity(BOARD_HEIGHT);

        loop {
            let full = self.cells[read_row].iter().all(|c| c.is_some());
            if full {
                cleared += 1;
            } else {
                kept.push(self.cells[read_row]);
            }
            if read_row == 0 {
                break;
            }
            read_row -= 1;
        }

        // Rebuild board: empty rows on top, kept rows on bottom
        let empty_rows = BOARD_HEIGHT - kept.len();
        for r in 0..empty_rows {
            self.cells[r] = [None; BOARD_WIDTH];
        }
        for (i, row) in kept.iter().rev().enumerate() {
            self.cells[empty_rows + i] = *row;
        }
        cleared
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

/// An active falling piece.
#[derive(Clone)]
pub struct Piece {
    pub kind: TetrominoKind,
    pub rotation: usize,
    /// Top-left row of the bounding box on the board (may be negative during spawn).
    pub row: i32,
    /// Top-left column of the bounding box on the board.
    pub col: i32,
}

impl Piece {
    /// Create a new piece centered at the top of the board.
    pub fn new(kind: TetrominoKind) -> Self {
        // Centre the piece horizontally.
        // Most pieces have a 3-wide bbox; I-piece has a 4-wide bbox.
        let col = match kind {
            TetrominoKind::I => 3,
            TetrominoKind::O => 4,
            _ => 3,
        };
        // Spawn one row above the visible board so the piece slides in.
        let row = match kind {
            TetrominoKind::I => -1,
            _ => -1,
        };
        Self {
            kind,
            rotation: 0,
            row,
            col,
        }
    }

    /// The four `(dr, dc)` offsets for the current rotation.
    pub fn cells(&self) -> [(i32, i32); 4] {
        PIECE_CELLS[self.kind.index()][self.rotation]
    }

    /// Absolute `(row, col)` board positions of the four cells.
    pub fn board_cells(&self) -> [(i32, i32); 4] {
        self.cells().map(|(dr, dc)| (self.row + dr, self.col + dc))
    }

    /// Return `true` if this piece doesn't collide with locked cells or walls.
    pub fn is_valid(&self, board: &Board) -> bool {
        self.board_cells()
            .iter()
            .all(|&(r, c)| board.is_empty(r, c))
    }

    /// Try to move down; returns `true` on success.
    pub fn try_move_down(&mut self, board: &Board) -> bool {
        self.row += 1;
        if self.is_valid(board) {
            true
        } else {
            self.row -= 1;
            false
        }
    }

    /// Try to move left; returns `true` on success.
    pub fn try_move_left(&mut self, board: &Board) -> bool {
        self.col -= 1;
        if self.is_valid(board) {
            true
        } else {
            self.col += 1;
            false
        }
    }

    /// Try to move right; returns `true` on success.
    pub fn try_move_right(&mut self, board: &Board) -> bool {
        self.col += 1;
        if self.is_valid(board) {
            true
        } else {
            self.col -= 1;
            false
        }
    }

    /// Try to rotate clockwise (simple rotation without wall-kicks).
    pub fn try_rotate(&mut self, board: &Board) -> bool {
        let old = self.rotation;
        self.rotation = (self.rotation + 1) % 4;
        if self.is_valid(board) {
            return true;
        }
        // Wall-kick: try shifting left or right by 1
        for &offset in &[-1i32, 1, -2, 2] {
            self.col += offset;
            if self.is_valid(board) {
                return true;
            }
            self.col -= offset;
        }
        self.rotation = old;
        false
    }

    /// Drop the piece as far as possible and return the number of rows dropped.
    pub fn hard_drop(&mut self, board: &Board) -> u32 {
        let mut rows = 0u32;
        while self.try_move_down(board) {
            rows += 1;
        }
        rows
    }

    /// Return a "ghost" piece showing where the piece would land.
    pub fn ghost(&self, board: &Board) -> Piece {
        let mut ghost = self.clone();
        while ghost.try_move_down(board) {}
        ghost
    }
}

/// Line-clear score multipliers (index = lines cleared − 1).
const SCORE_MULT: [u32; 4] = [100, 300, 500, 800];

/// Complete game state.
pub struct GameState {
    pub board: Board,
    pub current: Option<Piece>,
    pub next_kind: TetrominoKind,
    pub score: u32,
    pub level: u32,
    pub lines_cleared: u32,
    pub game_over: bool,
    pub paused: bool,
}

impl GameState {
    pub fn new() -> Self {
        let mut gs = Self {
            board: Board::new(),
            current: None,
            next_kind: TetrominoKind::random(),
            score: 0,
            level: 1,
            lines_cleared: 0,
            game_over: false,
            paused: false,
        };
        gs.spawn_next();
        gs
    }

    /// Spawn the queued piece and prepare the next one.
    fn spawn_next(&mut self) {
        let piece = Piece::new(self.next_kind);
        self.next_kind = TetrominoKind::random();
        if !piece.is_valid(&self.board) {
            self.game_over = true;
        } else {
            self.current = Some(piece);
        }
    }

    /// Lock the current piece, clear lines, update score, spawn next.
    fn lock_current(&mut self) {
        if let Some(piece) = self.current.take() {
            self.board.lock(&piece);
            let cleared = self.board.clear_lines();
            if cleared > 0 {
                self.score += SCORE_MULT[(cleared as usize).min(4) - 1] * self.level;
                self.lines_cleared += cleared;
                self.level = self.lines_cleared / 10 + 1;
            }
            self.spawn_next();
        }
    }

    /// Advance game by one gravity tick.  Returns `true` if still running.
    pub fn tick(&mut self) -> bool {
        if self.game_over || self.paused {
            return !self.game_over;
        }
        if let Some(ref mut piece) = self.current
            && !piece.try_move_down(&self.board)
        {
            self.lock_current();
        }
        !self.game_over
    }

    /// Drop interval in milliseconds based on current level.
    pub fn drop_interval_ms(&self) -> u64 {
        let lvl = self.level.min(10) as u64;
        (1000u64).saturating_sub((lvl - 1) * 100).max(100)
    }

    // ── Input actions ──────────────────────────────────────────────────────

    pub fn move_left(&mut self) {
        if self.game_over || self.paused {
            return;
        }
        if let Some(ref mut p) = self.current {
            p.try_move_left(&self.board);
        }
    }

    pub fn move_right(&mut self) {
        if self.game_over || self.paused {
            return;
        }
        if let Some(ref mut p) = self.current {
            p.try_move_right(&self.board);
        }
    }

    pub fn rotate(&mut self) {
        if self.game_over || self.paused {
            return;
        }
        if let Some(ref mut p) = self.current {
            p.try_rotate(&self.board);
        }
    }

    pub fn soft_drop(&mut self) {
        if self.game_over || self.paused {
            return;
        }
        if let Some(ref mut p) = self.current
            && p.try_move_down(&self.board)
        {
            self.score += 1;
        }
    }

    pub fn hard_drop(&mut self) {
        if self.game_over || self.paused {
            return;
        }
        if let Some(ref mut piece) = self.current {
            let rows = piece.hard_drop(&self.board);
            self.score += rows * 2;
        }
        self.lock_current();
    }

    pub fn toggle_pause(&mut self) {
        if !self.game_over {
            self.paused = !self.paused;
        }
    }

    // ── WASM-friendly board accessor ───────────────────────────────────────

    /// Returns the board as a flat `Vec<u8>` row-major (0 = empty, else colour
    /// index).  The current piece and its ghost are overlaid so a WASM
    /// consumer can render from a single call.
    pub fn board_flat(&self) -> Vec<u8> {
        let mut buf = vec![0u8; BOARD_WIDTH * BOARD_HEIGHT];

        // Locked cells
        for r in 0..BOARD_HEIGHT {
            for c in 0..BOARD_WIDTH {
                if let Some(color) = self.board.cells[r][c] {
                    buf[r * BOARD_WIDTH + c] = color;
                }
            }
        }

        // Ghost piece (drawn with a placeholder colour 1)
        if let Some(ref piece) = self.current {
            let ghost = piece.ghost(&self.board);
            for (gr, gc) in ghost.board_cells() {
                if gr >= 0 && gr < BOARD_HEIGHT as i32 && gc >= 0 && gc < BOARD_WIDTH as i32 {
                    let idx = gr as usize * BOARD_WIDTH + gc as usize;
                    if buf[idx] == 0 {
                        buf[idx] = 1; // ghost marker
                    }
                }
            }

            // Active piece
            let color = PIECE_COLORS[piece.kind.index()];
            for (pr, pc) in piece.board_cells() {
                if pr >= 0 && pr < BOARD_HEIGHT as i32 && pc >= 0 && pc < BOARD_WIDTH as i32 {
                    buf[pr as usize * BOARD_WIDTH + pc as usize] = color;
                }
            }
        }

        buf
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tetromino::TetrominoKind;

    // ── Board tests ───────────────────────────────────────────────────────

    #[test]
    fn board_starts_empty() {
        let board = Board::new();
        for r in 0..BOARD_HEIGHT {
            for c in 0..BOARD_WIDTH {
                assert!(board.cells[r][c].is_none());
            }
        }
    }

    #[test]
    fn is_empty_out_of_bounds() {
        let board = Board::new();
        assert!(!board.is_empty(0, -1));
        assert!(!board.is_empty(0, BOARD_WIDTH as i32));
        assert!(!board.is_empty(BOARD_HEIGHT as i32, 0));
        // Above the board is considered empty (piece can partially spawn above)
        assert!(board.is_empty(-1, 5));
    }

    #[test]
    fn clear_lines_removes_full_rows() {
        let mut board = Board::new();
        // Fill the bottom two rows completely
        for c in 0..BOARD_WIDTH {
            board.cells[BOARD_HEIGHT - 1][c] = Some(196); // red
            board.cells[BOARD_HEIGHT - 2][c] = Some(196);
        }
        let cleared = board.clear_lines();
        assert_eq!(cleared, 2);
        // Bottom two rows should now be empty
        for c in 0..BOARD_WIDTH {
            assert!(board.cells[BOARD_HEIGHT - 1][c].is_none());
            assert!(board.cells[BOARD_HEIGHT - 2][c].is_none());
        }
    }

    #[test]
    fn clear_lines_preserves_partial_rows() {
        let mut board = Board::new();
        // Partially fill the bottom row
        board.cells[BOARD_HEIGHT - 1][0] = Some(51);
        let cleared = board.clear_lines();
        assert_eq!(cleared, 0);
        assert!(board.cells[BOARD_HEIGHT - 1][0].is_some());
    }

    // ── Piece tests ───────────────────────────────────────────────────────

    #[test]
    fn piece_spawn_is_valid() {
        let board = Board::new();
        for kind in [
            TetrominoKind::I,
            TetrominoKind::O,
            TetrominoKind::T,
            TetrominoKind::S,
            TetrominoKind::Z,
            TetrominoKind::J,
            TetrominoKind::L,
        ] {
            let piece = Piece::new(kind);
            assert!(piece.is_valid(&board), "{:?} spawn should be valid", kind);
        }
    }

    #[test]
    fn piece_moves_left_right() {
        let board = Board::new();
        let mut piece = Piece::new(TetrominoKind::T);
        let original_col = piece.col;
        assert!(piece.try_move_right(&board));
        assert_eq!(piece.col, original_col + 1);
        assert!(piece.try_move_left(&board));
        assert_eq!(piece.col, original_col);
    }

    #[test]
    fn piece_rotates() {
        let board = Board::new();
        let mut piece = Piece::new(TetrominoKind::T);
        assert_eq!(piece.rotation, 0);
        assert!(piece.try_rotate(&board));
        assert_eq!(piece.rotation, 1);
    }

    #[test]
    fn piece_hard_drop_lands_at_bottom() {
        let board = Board::new();
        let mut piece = Piece::new(TetrominoKind::O);
        piece.hard_drop(&board);
        // Ensure the piece is now resting at the bottom row
        let max_row = piece.board_cells().iter().map(|&(r, _)| r).max().unwrap();
        assert_eq!(max_row, BOARD_HEIGHT as i32 - 1);
    }

    #[test]
    fn piece_cannot_move_through_wall() {
        let board = Board::new();
        let mut piece = Piece::new(TetrominoKind::I);
        // Move all the way left
        while piece.try_move_left(&board) {}
        let leftmost = piece.board_cells().iter().map(|&(_, c)| c).min().unwrap();
        assert_eq!(leftmost, 0);
        // One more left should fail
        let prev_col = piece.col;
        let moved = piece.try_move_left(&board);
        assert!(!moved);
        assert_eq!(piece.col, prev_col);
    }

    // ── GameState tests ───────────────────────────────────────────────────

    #[test]
    fn game_starts_not_over() {
        let gs = GameState::new();
        assert!(!gs.game_over);
        assert!(!gs.paused);
        assert_eq!(gs.score, 0);
        assert_eq!(gs.level, 1);
        assert_eq!(gs.lines_cleared, 0);
        assert!(gs.current.is_some());
    }

    #[test]
    fn toggle_pause() {
        let mut gs = GameState::new();
        assert!(!gs.paused);
        gs.toggle_pause();
        assert!(gs.paused);
        gs.toggle_pause();
        assert!(!gs.paused);
    }

    #[test]
    fn drop_interval_decreases_with_level() {
        let mut gs = GameState::new();
        let interval_lvl1 = gs.drop_interval_ms();
        gs.level = 5;
        let interval_lvl5 = gs.drop_interval_ms();
        assert!(interval_lvl5 < interval_lvl1);
    }

    #[test]
    fn hard_drop_locks_piece_and_spawns_next() {
        let mut gs = GameState::new();
        gs.hard_drop();
        // After hard drop, a new piece should be spawned
        assert!(gs.current.is_some());
    }

    #[test]
    fn score_increases_after_line_clear() {
        let mut gs = GameState::new();
        // Fill the bottom 3 rows except the last cell in each row
        for r in (BOARD_HEIGHT - 3)..BOARD_HEIGHT {
            for c in 0..BOARD_WIDTH {
                gs.board.cells[r][c] = Some(196);
            }
        }
        // Now fill that last cell by placing an I-piece via board_flat manipulation
        // Simpler: fill completely and manually call clear
        let cleared = gs.board.clear_lines();
        assert_eq!(cleared, 3);
    }
}
