use rand::Rng;

/// The seven standard Tetris pieces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TetrominoKind {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl TetrominoKind {
    /// Choose a random tetromino kind.
    pub fn random() -> Self {
        let mut rng = rand::rng();
        match rng.random_range(0..7_u8) {
            0 => Self::I,
            1 => Self::O,
            2 => Self::T,
            3 => Self::S,
            4 => Self::Z,
            5 => Self::J,
            _ => Self::L,
        }
    }

    /// Index into `PIECE_CELLS` for this kind.
    pub fn index(self) -> usize {
        self as usize
    }
}

/// Cell offsets `(row, col)` within a piece's bounding box for all 7 kinds × 4
/// rotations.  The bounding box is at most 4×4.
///
/// Layout: `PIECE_CELLS[kind_index][rotation]` → 4 `(row, col)` pairs.
pub const PIECE_CELLS: [[[(i32, i32); 4]; 4]; 7] = [
    // 0 – I  (4×4 bounding box)
    [
        [(1, 0), (1, 1), (1, 2), (1, 3)], // 0° – horizontal
        [(0, 2), (1, 2), (2, 2), (3, 2)], // 90° – vertical
        [(2, 0), (2, 1), (2, 2), (2, 3)], // 180°
        [(0, 1), (1, 1), (2, 1), (3, 1)], // 270°
    ],
    // 1 – O  (2×2 bounding box, rotation has no effect)
    [
        [(0, 0), (0, 1), (1, 0), (1, 1)],
        [(0, 0), (0, 1), (1, 0), (1, 1)],
        [(0, 0), (0, 1), (1, 0), (1, 1)],
        [(0, 0), (0, 1), (1, 0), (1, 1)],
    ],
    // 2 – T  (3×3 bounding box)
    [
        [(0, 1), (1, 0), (1, 1), (1, 2)], // 0°  bump up
        [(0, 0), (1, 0), (1, 1), (2, 0)], // 90°  bump right
        [(1, 0), (1, 1), (1, 2), (2, 1)], // 180° bump down
        [(0, 1), (1, 0), (1, 1), (2, 1)], // 270° bump left
    ],
    // 3 – S  (3×3 bounding box)
    [
        [(0, 1), (0, 2), (1, 0), (1, 1)], // 0°
        [(0, 0), (1, 0), (1, 1), (2, 1)], // 90°
        [(0, 1), (0, 2), (1, 0), (1, 1)], // 180° (same as 0°)
        [(0, 0), (1, 0), (1, 1), (2, 1)], // 270° (same as 90°)
    ],
    // 4 – Z  (3×3 bounding box)
    [
        [(0, 0), (0, 1), (1, 1), (1, 2)], // 0°
        [(0, 1), (1, 0), (1, 1), (2, 0)], // 90°
        [(0, 0), (0, 1), (1, 1), (1, 2)], // 180° (same as 0°)
        [(0, 1), (1, 0), (1, 1), (2, 0)], // 270° (same as 90°)
    ],
    // 5 – J  (3×3 bounding box)
    [
        [(0, 0), (1, 0), (1, 1), (1, 2)], // 0°  – J flat, blue square top-left
        [(0, 0), (0, 1), (1, 0), (2, 0)], // 90°
        [(1, 0), (1, 1), (1, 2), (2, 2)], // 180°
        [(0, 1), (1, 1), (2, 0), (2, 1)], // 270°
    ],
    // 6 – L  (3×3 bounding box)
    [
        [(0, 2), (1, 0), (1, 1), (1, 2)], // 0°  – L flat, orange square top-right
        [(0, 0), (1, 0), (2, 0), (2, 1)], // 90°
        [(1, 0), (1, 1), (1, 2), (2, 0)], // 180°
        [(0, 0), (0, 1), (1, 1), (2, 1)], // 270°
    ],
];
