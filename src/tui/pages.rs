pub mod branches;
pub mod cadence;
pub mod home;
pub mod silo;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum Page {
    #[default]
    Home = 0,
    Cadence = 1,
    Silo = 2,
    Branches = 3,
}

impl Page {
    pub const ALL: [Page; 4] = [Page::Home, Page::Cadence, Page::Silo, Page::Branches];

    pub fn to_str(&self) -> &'static str {
        match self {
            Page::Home => "Home",
            Page::Cadence => "Cadence",
            Page::Silo => "Silo",
            Page::Branches => "Branches",
        }
    }

    pub fn size() -> usize {
        Self::ALL.len()
    }

    pub fn next(&self) -> Page {
        match &self {
            Page::Home => Page::Cadence,
            Page::Cadence => Page::Silo,
            Page::Silo => Page::Branches,
            Page::Branches => Page::Home,
        }
    }
}
