use std::io::{self, BufWriter};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;

use rust_tetris::{game::GameState, render};

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    terminal::enable_raw_mode()?;
    render::setup_terminal(&mut out)?;
    render::draw_static_frame(&mut out)?;

    let mut gs = GameState::new();
    let mut last_tick = Instant::now();

    loop {
        let interval = Duration::from_millis(gs.drop_interval_ms());
        let timeout = interval.saturating_sub(last_tick.elapsed());

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(KeyEvent {
                    code, modifiers, ..
                }) => match code {
                    KeyCode::Left => gs.move_left(),
                    KeyCode::Right => gs.move_right(),
                    KeyCode::Up | KeyCode::Char('z') | KeyCode::Char('Z') => gs.rotate(),
                    KeyCode::Down => gs.soft_drop(),
                    KeyCode::Char(' ') => gs.hard_drop(),
                    KeyCode::Char('p') | KeyCode::Char('P') => gs.toggle_pause(),
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => break,
                    _ => {}
                },
                Event::Resize(_, _) => {
                    render::clear_and_redraw_frame(&mut out)?;
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= interval {
            gs.tick();
            last_tick = Instant::now();
        }

        render::render(&mut out, &gs)?;

        if gs.game_over {
            // Wait for a final key before quitting
            loop {
                if let Event::Key(KeyEvent {
                    code: KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc,
                    ..
                }) = event::read()?
                {
                    break;
                }
            }
            break;
        }
    }

    render::teardown_terminal(&mut out)?;
    terminal::disable_raw_mode()?;
    println!("Thanks for playing rust-tetris!");
    Ok(())
}
