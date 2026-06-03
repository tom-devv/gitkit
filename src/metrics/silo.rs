use crate::{git::kit::KitRepo, tui::Renderable};

pub struct SiloData {}

impl SiloData {
    pub fn new(repo: &KitRepo) -> Self {
        Self {}
    }
}

pub struct SiloPage {
    data: SiloData,
}

impl SiloPage {
    pub fn new(data: SiloData) -> Self {
        Self { data }
    }
}

impl Renderable for SiloPage {
    fn render(&mut self, frame: &mut ratatui::prelude::Frame, _area: ratatui::prelude::Rect) {}
}
