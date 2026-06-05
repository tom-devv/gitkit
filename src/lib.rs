use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use git2::Binding;
use std::{io, panic, time::Duration};

use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{
    error::Result,
    git::kit::KitRepo,
    metrics::silo::SiloData,
    tui::{state::TuiState, ui::render},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct GKitArgs {
    #[arg(default_value = ".")]
    target_path: String,

    #[arg(long, hide = true)]
    pub debug: bool,
}

pub mod error;
pub mod git;
pub mod metrics;
pub mod tui;

pub fn run(args: GKitArgs) -> Result<()> {
    let repo = KitRepo::open(args.target_path)?;

    if args.debug {
        println!("Debug Mode\n");
        let churn = SiloData::get_churn(&repo)?;

        return Ok(());
    }
    let mut state = TuiState::new(&repo)?; // blocking

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    terminal::enable_raw_mode()?;
    ratatui::crossterm::execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    panic::set_hook(Box::new(move |panic| {
        let _ = terminal::disable_raw_mode();
        let _ =
            ratatui::crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        eprintln!("Panic??: {}", panic);
    }));

    terminal.hide_cursor()?;
    terminal.clear()?;

    let tui_result = tui(&mut terminal, &mut state, &repo);

    let _ = terminal.show_cursor();
    let _ = ratatui::crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    let _ = terminal::disable_raw_mode();

    tui_result // returns once drawing stops
}

pub fn tui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut TuiState,
    repo: &KitRepo,
) -> Result<()> {
    while !state.is_quit {
        terminal.draw(|frame| render(frame, state))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    state.handle_key_event(key, repo);
                }
            }
        }
    }

    Ok(())
}
