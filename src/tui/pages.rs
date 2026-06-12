

pub mod cadence;
pub mod home;
pub mod silo;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum Page {
    #[default]
    Home = 0,
    Cadence = 1,
    Silo = 2,
}

impl Page {
    pub const ALL: [Page; 3] = [Page::Home, Page::Cadence, Page::Silo];

    pub fn to_str(&self) -> &'static str {
        match self {
            Page::Home => "Home",
            Page::Cadence => "Cadence",
            Page::Silo => "Silo",
        }
    }

    pub fn size() -> usize {
        Self::ALL.len()
    }

    pub fn next(&self) -> Page {
        match &self {
            Page::Home => Page::Cadence,
            Page::Cadence => Page::Silo,
            Page::Silo => Page::Home,
        }
    }
}

