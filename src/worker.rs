use std::{sync::mpsc, thread};

use crate::{
    git::kit::KitRepo,
    git::metrics::{cadence::CadenceData, home::HomeData, silo::SiloData},
};

#[derive(Clone, Copy)]
enum MetricTask {
    Home,
    Cadence,
    Silo,
}

const METRIC_TASKS: [MetricTask; 3] = [MetricTask::Home, MetricTask::Cadence, MetricTask::Silo];

pub enum DataUpdate {
    Home(HomeData),
    Cadence(CadenceData),
    Silo(SiloData),
}
#[derive(Clone, Copy)]
pub enum WorkerCommand {
    Refresh,
    Quit,
}

pub struct Worker {
    cmd_senders: Vec<mpsc::Sender<WorkerCommand>>,
    pub update_rx: mpsc::Receiver<DataUpdate>,
}

impl Worker {
    pub fn start(repo_path: std::path::PathBuf) -> Self {
        let mut cmd_senders: Vec<mpsc::Sender<WorkerCommand>> = Vec::new();
        let (update_tx, update_rx) = mpsc::channel::<DataUpdate>();

        for task in METRIC_TASKS {
            let (cmd_tx, cmd_rx) = mpsc::channel::<WorkerCommand>();
            let thread_update_tx = update_tx.clone();
            let thread_path = repo_path.clone();

            thread::spawn(move || {
                let repo = KitRepo::open(&thread_path).unwrap();

                while let Ok(cmd) = cmd_rx.recv() {
                    match cmd {
                        WorkerCommand::Refresh => {
                            let update = match task {
                                MetricTask::Home => DataUpdate::Home(HomeData::new(&repo)),
                                MetricTask::Cadence => DataUpdate::Cadence(CadenceData::new(&repo)),
                                MetricTask::Silo => DataUpdate::Silo(SiloData::new(&repo)),
                            };

                            let _ = thread_update_tx.send(update);
                        }
                        WorkerCommand::Quit => break,
                    }
                }
            });
            cmd_senders.push(cmd_tx);
        }

        Self {
            cmd_senders,
            update_rx,
        }
    }

    pub fn refresh(&self) {
        for tx in &self.cmd_senders {
            let _ = tx.send(WorkerCommand::Refresh);
        }
    }

    pub fn quit(&self) {
        for tx in &self.cmd_senders {
            let _ = tx.send(WorkerCommand::Quit);
        }
    }
}
