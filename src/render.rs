use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};

use crate::game::{GameState, BOARD_HEIGHT, BOARD_WIDTH};
use crate::tetromino::{TetrominoKind, PIECE_CELLS};

// ── Layout constants ──────────────────────────────────────────────────────────

/// Each board cell is drawn as two characters wide so pieces look square.
const CELL_W: u16 = 2;

/// Column (0-based terminal column) where the board's left inner edge starts
/// (relative to the UI origin).
const BOARD_LEFT: u16 = 1; // 1 char for left border

/// Column where the sidebar starts (right of board + right border + gap),
/// relative to the UI origin.
const SIDE_LEFT: u16 = BOARD_LEFT + BOARD_WIDTH as u16 * CELL_W + 1 + 2;

/// Top row (0-based terminal row) of the board, relative to the UI origin.
const BOARD_TOP: u16 = 1;

/// Width of the sidebar box in terminal columns (matches the drawn box "╔══════════╗").
const SIDEBAR_WIDTH: u16 = 12;

/// Total columns the UI occupies (board + gap + sidebar box).
const TOTAL_COLS: u16 = SIDE_LEFT + SIDEBAR_WIDTH;

/// Total rows the UI occupies (controls label ends at row 24).
const TOTAL_ROWS: u16 = 25;

/// Compute the top-left (col, row) origin so the UI is centred in the terminal.
fn compute_origin() -> (u16, u16) {
    let (term_cols, term_rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let col = term_cols.saturating_sub(TOTAL_COLS) / 2;
    let row = term_rows.saturating_sub(TOTAL_ROWS) / 2;
    (col, row)
}

// ── Colour helpers ────────────────────────────────────────────────────────────

/// Map ANSI-256 index to the nearest crossterm `Color`.
fn ansi_to_color(n: u8) -> Color {
    match n {
        51 => Color::Cyan,       // I
        226 => Color::Yellow,    // O
        129 => Color::Magenta,   // T
        46 => Color::Green,      // S
        196 => Color::Red,       // Z
        21 => Color::Blue,       // J
        208 => Color::DarkYellow, // L (closest to orange in named colours)
        _ => Color::White,
    }
}

fn piece_color(kind: TetrominoKind) -> Color {
    match kind {
        TetrominoKind::I => Color::Cyan,
        TetrominoKind::O => Color::Yellow,
        TetrominoKind::T => Color::Magenta,
        TetrominoKind::S => Color::Green,
        TetrominoKind::Z => Color::Red,
        TetrominoKind::J => Color::Blue,
        TetrominoKind::L => Color::DarkYellow,
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Draw the initial static frame (borders, labels).
pub fn draw_static_frame(out: &mut impl Write) -> io::Result<()> {
    let (oc, or) = compute_origin();

    // ── Board border ──────────────────────────────────────────────────────
    // Top line
    queue!(
        out,
        MoveTo(oc, or),
        Print("╔"),
        Print("══".repeat(BOARD_WIDTH)),
        Print("╗"),
    )?;

    // Side borders for each row
    for r in 0..BOARD_HEIGHT as u16 {
        queue!(
            out,
            MoveTo(oc, or + BOARD_TOP + r),
            Print("║"),
            MoveTo(oc + BOARD_LEFT + BOARD_WIDTH as u16 * CELL_W, or + BOARD_TOP + r),
            Print("║"),
        )?;
    }

    // Bottom line
    queue!(
        out,
        MoveTo(oc, or + BOARD_TOP + BOARD_HEIGHT as u16),
        Print("╚"),
        Print("══".repeat(BOARD_WIDTH)),
        Print("╝"),
    )?;

    // ── Sidebar ───────────────────────────────────────────────────────────
    let sl = oc + SIDE_LEFT;

    // NEXT box
    queue!(out, MoveTo(sl, or), Print("╔══════════╗"))?;
    queue!(out, MoveTo(sl, or + 1), Print("║  NEXT    ║"))?;
    queue!(out, MoveTo(sl, or + 2), Print("╠══════════╣"))?;
    for r in 3..7u16 {
        queue!(out, MoveTo(sl, or + r), Print("║          ║"))?;
    }
    queue!(out, MoveTo(sl, or + 7), Print("╠══════════╣"))?;

    // SCORE box
    queue!(out, MoveTo(sl, or + 8), Print("║ SCORE    ║"))?;
    queue!(out, MoveTo(sl, or + 9), Print("║          ║"))?;
    queue!(out, MoveTo(sl, or + 10), Print("╠══════════╣"))?;

    // LEVEL box
    queue!(out, MoveTo(sl, or + 11), Print("║ LEVEL    ║"))?;
    queue!(out, MoveTo(sl, or + 12), Print("║          ║"))?;
    queue!(out, MoveTo(sl, or + 13), Print("╠══════════╣"))?;

    // LINES box
    queue!(out, MoveTo(sl, or + 14), Print("║ LINES    ║"))?;
    queue!(out, MoveTo(sl, or + 15), Print("║          ║"))?;
    queue!(out, MoveTo(sl, or + 16), Print("╚══════════╝"))?;

    // Controls help
    queue!(out, MoveTo(sl, or + 18), Print("Controls:"))?;
    queue!(out, MoveTo(sl, or + 19), Print("← → : Move"))?;
    queue!(out, MoveTo(sl, or + 20), Print("↑ / Z: Rotate"))?;
    queue!(out, MoveTo(sl, or + 21), Print("↓ : Soft Drop"))?;
    queue!(out, MoveTo(sl, or + 22), Print("SPC: Hard Drop"))?;
    queue!(out, MoveTo(sl, or + 23), Print("P : Pause"))?;
    queue!(out, MoveTo(sl, or + 24), Print("Q : Quit"))?;

    out.flush()
}

/// Render the dynamic parts of the game (board contents + sidebar values).
pub fn render(out: &mut impl Write, gs: &GameState) -> io::Result<()> {
    let origin = compute_origin();
    draw_board(out, gs, origin)?;
    draw_sidebar(out, gs, origin)?;
    if gs.paused {
        draw_overlay(out, "  PAUSED  ", "Press P to resume", origin)?;
    } else if gs.game_over {
        draw_overlay(out, " GAME OVER", "Press Q to quit", origin)?;
    }
    out.flush()
}

// ── Board drawing ─────────────────────────────────────────────────────────────

fn draw_board(out: &mut impl Write, gs: &GameState, (oc, or): (u16, u16)) -> io::Result<()> {
    // Pre-compute ghost cells
    let ghost_cells: Vec<(i32, i32)> = if let Some(ref piece) = gs.current {
        piece.ghost(&gs.board).board_cells().to_vec()
    } else {
        vec![]
    };

    // Pre-compute active piece cells + colour
    let (active_cells, active_color): (Vec<(i32, i32)>, Option<Color>) =
        if let Some(ref piece) = gs.current {
            (piece.board_cells().to_vec(), Some(piece_color(piece.kind)))
        } else {
            (vec![], None)
        };

    for r in 0..BOARD_HEIGHT {
        let term_row = or + BOARD_TOP + r as u16;
        queue!(out, MoveTo(oc + BOARD_LEFT, term_row))?;

        for c in 0..BOARD_WIDTH {
            let ri = r as i32;
            let ci = c as i32;

            if let Some(color) = active_color
                && active_cells.contains(&(ri, ci))
            {
                queue!(
                    out,
                    SetBackgroundColor(color),
                    SetForegroundColor(Color::Black),
                    Print("  "),
                    ResetColor,
                )?;
                continue;
            }

            if ghost_cells.contains(&(ri, ci)) {
                queue!(
                    out,
                    SetForegroundColor(Color::DarkGrey),
                    Print("░░"),
                    ResetColor,
                )?;
                continue;
            }

            if let Some(color) = gs.board.cells[r][c] {
                queue!(
                    out,
                    SetBackgroundColor(ansi_to_color(color)),
                    Print("  "),
                    ResetColor,
                )?;
            } else {
                queue!(out, Print("  "))?;
            }
        }
    }
    Ok(())
}

// ── Sidebar drawing ───────────────────────────────────────────────────────────

fn draw_sidebar(out: &mut impl Write, gs: &GameState, (oc, or): (u16, u16)) -> io::Result<()> {
    let sl = oc + SIDE_LEFT;

    // Next piece preview (4×4 area inside the box, rows 3..6)
    for r in 0..4u16 {
        queue!(out, MoveTo(sl + 1, or + 3 + r), Print("          "))?;
    }
    let next_color = piece_color(gs.next_kind);
    for (dr, dc) in PIECE_CELLS[gs.next_kind.index()][0] {
        let pr = or + 3 + dr as u16;
        let pc = sl + 1 + dc as u16 * CELL_W;
        queue!(
            out,
            MoveTo(pc, pr),
            SetBackgroundColor(next_color),
            Print("  "),
            ResetColor,
        )?;
    }

    // Score
    queue!(
        out,
        MoveTo(sl + 2, or + 9),
        Print(format!("{:<8}", gs.score)),
    )?;

    // Level
    queue!(
        out,
        MoveTo(sl + 2, or + 12),
        Print(format!("{:<8}", gs.level)),
    )?;

    // Lines
    queue!(
        out,
        MoveTo(sl + 2, or + 15),
        Print(format!("{:<8}", gs.lines_cleared)),
    )?;

    Ok(())
}

// ── Overlay ───────────────────────────────────────────────────────────────────

fn draw_overlay(out: &mut impl Write, title: &str, subtitle: &str, (oc, or): (u16, u16)) -> io::Result<()> {
    let center_row = or + BOARD_TOP + BOARD_HEIGHT as u16 / 2 - 1;
    let center_col = oc + BOARD_LEFT;

    queue!(
        out,
        MoveTo(center_col, center_row),
        SetBackgroundColor(Color::DarkRed),
        SetForegroundColor(Color::White),
        Print(format!(" {:^18} ", title)),
        ResetColor,
    )?;
    queue!(
        out,
        MoveTo(center_col, center_row + 1),
        SetBackgroundColor(Color::DarkRed),
        SetForegroundColor(Color::White),
        Print(format!(" {:^18} ", subtitle)),
        ResetColor,
    )?;
    Ok(())
}

/// Clear the terminal and hide the cursor before starting.
pub fn setup_terminal(out: &mut impl Write) -> io::Result<()> {
    execute!(
        out,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::cursor::Hide,
        Clear(ClearType::All),
    )
}

/// Restore the terminal to its normal state.
pub fn teardown_terminal(out: &mut impl Write) -> io::Result<()> {
    execute!(
        out,
        crossterm::cursor::Show,
        ResetColor,
        crossterm::terminal::LeaveAlternateScreen,
    )
}
