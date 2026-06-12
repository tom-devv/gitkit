use ratatui::style::Color;

pub mod pages;
pub mod state;
pub mod ui;
pub mod widgets;

pub const GRAY_BORDER_COLOR: ratatui::prelude::Color = Color::Rgb(105, 103, 97);
pub const PRIMARY: ratatui::prelude::Color = Color::White;
pub const ACCENT: ratatui::prelude::Color = Color::Rgb(220, 138, 120);
pub const ACCENT_TEXT: ratatui::prelude::Color = Color::Rgb(204, 208, 218);

//https://patorjk.com/software/taag/#p=display&f=ANSI+Shadow&t=gitkit&x=none&v=4&h=4&w=80&we=false
const GITKIT_ASCII: &str = r"

 ██████╗ ██╗████████╗██╗  ██╗██╗████████╗
██╔════╝ ██║╚══██╔══╝██║ ██╔╝██║╚══██╔══╝
██║  ███╗██║   ██║   █████╔╝ ██║   ██║   
██║   ██║██║   ██║   ██╔═██╗ ██║   ██║   
╚██████╔╝██║   ██║   ██║  ██╗██║   ██║   
 ╚═════╝ ╚═╝   ╚═╝   ╚═╝  ╚═╝╚═╝   ╚═╝   
                                                                                                                               
";
pub trait Renderable {
    fn render(&mut self, frame: &mut ratatui::prelude::Frame, area: ratatui::prelude::Rect);
}
