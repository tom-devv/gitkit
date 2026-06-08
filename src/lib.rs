use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, panic, time::Duration};

use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{
    error::Result,
    git::kit::KitRepo,
    tui::{state::TuiState, ui::render},
    worker::Worker,
};

pub mod error;
pub mod git;
pub mod metrics;
pub mod tui;
pub mod worker;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct GKitArgs {
    #[arg(default_value = ".")]
    target_path: String,

    #[arg(long, hide = true)]
    pub debug: bool,
}

pub fn run(args: GKitArgs) -> Result<()> {
    let repo = KitRepo::open(args.target_path)?;

    if args.debug {
        println!("Debug Mode\n");
        let _x = repo.list_branch();
        // println!("{:?}", x.len());

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

pub fn tui<'a>(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut TuiState,
    repo: &'a KitRepo,
) -> Result<()> {
    let repo_path = repo.inner.path().to_path_buf();

    let worker = Worker::start(repo_path);

    while !state.is_quit {
        terminal.draw(|frame| render(frame, state))?;

        if state.refresh {
            state.refresh = false;
            state.loading = true;
            worker.trigger_refresh();
        }

        // when chan gets message do refresh
        if let Ok(payload) = worker.payload_rx.try_recv() {
            state.refresh(payload);
        }

        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        println!("pressed \n x");
                        state.handle_key_event(key, repo);
                    }
                }
                Event::Mouse(mouse) => {
                    state.handle_mouse_event(mouse, repo);
                }
                _ => {}
            }
        }
    }

    worker.quit();

    Ok(())
}
