use std::env;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Tabs, TitlePosition},
};

use crate::tui::{ACCENT, ACCENT_TEXT, GRAY_BORDER_COLOR, Renderable, page::Page, state::TuiState};

pub fn render(frame: &mut Frame, state: &mut TuiState) {
    let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]) // 3 chars is enough to render the nav text
        .margin(2)
        .split(frame.area());

    render_tabs(frame, &state, chunks[0]);

    let content_area = render_outer_frame(frame, chunks[1], state);

    match state.active_page {
        Page::Home => state.home.render(frame, content_area),
        Page::Cadence => state.cadence.render(frame, content_area),
        Page::Todo => {}
        _ => {}
    }
}

fn render_outer_frame(frame: &mut Frame, chunk: Rect, state: &TuiState) -> Rect {
    let border = Block::bordered()
        .title(format_keybinds(state))
        .title_alignment(ratatui::layout::HorizontalAlignment::Center)
        .title_position(TitlePosition::Bottom)
        .border_type(BorderType::Plain)
        .border_style(GRAY_BORDER_COLOR)
        .padding(Padding::ZERO);

    frame.render_widget(&border, chunk);
    border.inner(chunk)
}

// keybind code from binsider
// https://github.com/orhun/binsider/blob/ebbb36aeff178b42fb81911cc697173ec6d21972/src/tui/ui.rs#L174
fn format_keybinds(state: &TuiState) -> Line<'_> {
    let key_bindings = state.get_binds();

    Line::from(
        key_bindings
            .iter()
            .enumerate()
            .flat_map(|(i, (keys, desc))| {
                vec![
                    "[".fg(Color::Rgb(100, 100, 100)),
                    (format!("{} ", keys)).fg(ACCENT), // adds a space
                    "→ ".fg(Color::Rgb(100, 100, 100)),
                    Span::from(*desc).fg(ACCENT_TEXT),
                    "]".fg(Color::Rgb(100, 100, 100)),
                    if i != key_bindings.len() - 1 { " " } else { "" }.into(),
                ]
            })
            .collect::<Vec<Span>>(),
    )
}

fn render_tabs(frame: &mut Frame, state: &TuiState, chunk: Rect) {
    let title = format!(
        "|< {} (v{}) >|",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    let nav_block = Block::bordered()
        .border_style(GRAY_BORDER_COLOR)
        .title(title)
        .title_style(Color::White)
        .title_alignment(ratatui::layout::HorizontalAlignment::Center);

    let nav_tabs = nav(state).block(nav_block);

    frame.render_widget(nav_tabs, chunk);
}

pub fn nav(state: &TuiState) -> Tabs<'static> {
    let tab_titles = Page::ALL.iter().map(|page| page.to_str());
    let tabs = Tabs::new(tab_titles)
        .select(state.active_page as usize)
        .style(Style::default().fg(ACCENT_TEXT))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(ACCENT));

    tabs
}

pub fn draw_placeholder(frame: &mut Frame, area: Rect, label: &str, color: Color) {
    let placeholder = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color))
        .title(format!(" {} ({}x{}) ", label, area.width, area.height))
        .title_style(Style::default().fg(color));

    frame.render_widget(placeholder, area);
}
