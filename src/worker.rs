use std::{sync::mpsc, thread};

use crate::{
    git::kit::KitRepo,
    metrics::{cadence::CadenceData, silo::SiloData},
    tui::page::HomeData,
};

pub struct DataPayload {
    pub home_data: HomeData,
    pub cadence_data: CadenceData,
    pub silo_data: SiloData,
}

pub enum WorkerCommand {
    Refresh,
    Quit,
}

pub struct Worker {
    cmd_tx: mpsc::Sender<WorkerCommand>,
    pub payload_rx: mpsc::Receiver<DataPayload>,
}

impl Worker {
    pub fn start(repo_path: std::path::PathBuf) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel::<WorkerCommand>();
        let (payload_tx, payload_rx) = mpsc::channel::<DataPayload>();

        thread::spawn(move || {
            let thread_repo = KitRepo::open(&repo_path).unwrap();

            // blocking check for msgs
            while let Ok(cmd) = cmd_rx.recv() {
                match cmd {
                    WorkerCommand::Refresh => {
                        let payload = DataPayload {
                            home_data: HomeData::new(&thread_repo),
                            cadence_data: CadenceData::new(&thread_repo),
                            silo_data: SiloData::new(&thread_repo),
                        };
                        let _ = payload_tx.send(payload); // send new data
                    }
                    WorkerCommand::Quit => break,
                }
            }
        });

        Self { cmd_tx, payload_rx }
    }

    // send refresh msg
    pub fn refresh(&self) {
        let _ = self.cmd_tx.send(WorkerCommand::Refresh);
    }

    pub fn quit(&self) {
        let _ = self.cmd_tx.send(WorkerCommand::Quit);
    }
}
