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
    tui::{
        search::Search,
        state::{
            Mode::{Normal, Searching},
            TuiState,
        },
        ui::render,
    },
    worker::Worker,
};

pub mod error;
pub mod git;
pub mod tui;
pub mod worker;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct KitArgs {
    #[arg(default_value = ".")]
    target_path: String,

    #[arg(long, hide = true)]
    pub debug: bool,
}

pub fn run(args: KitArgs) -> Result<()> {
    let repo = KitRepo::open(&args.target_path)?;

    if args.debug {
        println!("Debug Mode\n");

        return Ok(());
    }

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

    let tui_result = tui(&mut terminal, &repo, &args);

    let _ = terminal.show_cursor();
    let _ = ratatui::crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    let _ = terminal::disable_raw_mode();

    tui_result // returns once drawing stops
}

pub fn tui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    repo: &KitRepo,
    args: &KitArgs,
) -> Result<()> {
    let repo_path = repo.inner.path().to_path_buf();

    let worker = Worker::start(repo_path);
    worker.refresh();

    let mut state = TuiState::new();

    while !state.is_quit && !args.debug {
        terminal.draw(|frame| render(frame, &mut state))?;

        if state.refresh {
            state.refresh();
            worker.refresh();
        }

        // when update chan gets message update state
        if let Ok(update) = worker.update_rx.try_recv() {
            state.update(update);
        }

        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) => match state.mode {
                    Normal => {
                        if key.kind == KeyEventKind::Press {
                            state.handle_key_event(key, repo);
                        }
                    }
                    Searching => Search::handle_event(&mut state, &key),
                },
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
