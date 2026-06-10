use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, panic, sync::mpsc::TryRecvError, time::Duration};

use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{
    error::Result,
    git::kit::KitRepo,
    tui::{
        state::TuiState,
        ui::render,
        widget::{LoadingState, LoadingWidget},
    },
    worker::{DataPayload, Worker},
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
    // let mut state = TuiState::new(&repo)?; // blocking

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

    let tui_result = tui(&mut terminal, &repo);

    let _ = terminal.show_cursor();
    let _ = ratatui::crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    let _ = terminal::disable_raw_mode();

    tui_result // returns once drawing stops
}

pub fn tui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    // state: &mut TuiState,
    repo: &KitRepo,
) -> Result<()> {
    let repo_path = repo.inner.path().to_path_buf();

    let worker = Worker::start(repo_path);
    worker.refresh();

    // listen for woker message that data has been fetched
    let data = initial_fetch(&worker, terminal)?;
    let mut state = TuiState::new(data)?;

    while !state.is_quit {
        terminal.draw(|frame| render(frame, &mut state))?;

        if state.refresh {
            state.refresh = false;
            state.loading = true;
            worker.refresh();
        }

        // when chan gets message do refresh
        if let Ok(payload) = worker.payload_rx.try_recv() {
            state.refresh(payload);
        }

        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
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

// this could be nicer with a refactor of Worker
fn initial_fetch(
    worker: &Worker,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<DataPayload> {
    let mut loading_state = LoadingState::new();

    let data = loop {
        match worker.payload_rx.try_recv() {
            Ok(msg) => break msg,

            // show loading widget until data is fetched
            Err(TryRecvError::Empty) => {
                terminal.draw(|frame| {
                    frame.render_stateful_widget(
                        LoadingWidget::default(),
                        frame.area(),
                        &mut loading_state,
                    )
                })?;
            }

            Err(TryRecvError::Disconnected) => panic!("Failed to fetch data"),
        }

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    panic!("Quitting"); // state does not exist here so we must panic to exit
                }
            }
        }
    };
    Ok(data)
}
