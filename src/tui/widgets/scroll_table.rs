use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState},
};

pub struct ScrollingTable<'table> {
    table: Table<'table>,
}

impl<'table> StatefulWidget for ScrollingTable<'table> {
    type State = ScrollingTableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        <Table as StatefulWidget>::render(self.table, area, buf, &mut state.table_state);
        Self::render_scrollbar(buf, area, state);
    }
}

impl<'table> ScrollingTable<'table> {
    pub fn new(table: Table<'table>) -> Self {
        Self { table }
    }

    fn render_scrollbar(buf: &mut Buffer, area: Rect, state: &mut ScrollingTableState) {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        state.scroll_state = state
            .scroll_state
            .viewport_content_length(area.height as usize);

        let scroll_area = area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        });

        scrollbar.render(
            scroll_area,
            buf,
            &mut state.scroll_state.position(state.selected_index),
        );
    }
}

#[derive(Default, Debug)]
pub struct ScrollingTableState {
    pub data_len: usize,
    pub scroll_state: ScrollbarState,
    pub table_state: TableState,
    pub selected_index: usize,
    indices: Vec<usize>,
}

impl ScrollingTableState {
    pub fn new(data_len: usize) -> Self {
        Self {
            data_len,
            scroll_state: ScrollbarState::new(data_len).position(0),
            table_state: TableState::default().with_selected(0),
            selected_index: 0,
            indices: (0..data_len).collect(),
        }
    }

    pub fn apply_search<T, F>(&mut self, items: &[T], mut is_match: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.indices = items
            .iter()
            .enumerate()
            .filter(|(_, item)| is_match(*item))
            .map(|(index, _)| index)
            .collect();

        self.data_len = self.indices.len();
        self.select_index(0);
    }

    pub fn get_selected<'a, T>(&self, items: &'a [T]) -> Option<&'a T> {
        let original_index = self.indices.get(self.selected_index)?;
        items.get(*original_index)
    }

    pub fn iter_visible<'a, T>(&'a self, items: &'a [T]) -> impl Iterator<Item = &'a T> {
        self.indices.iter().filter_map(move |&i| items.get(i))
    }

    fn select_index(&mut self, index: usize) {
        self.selected_index = index;
        self.table_state.select(Some(self.selected_index));
        self.scroll_state = self.scroll_state.position(self.selected_index);
    }

    pub fn handle_scroll(&mut self, key_event: &KeyEvent) {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => self.next(1),
            (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => self.prev(1),

            (KeyCode::Char('g'), KeyModifiers::NONE) => self.top(),
            // G (with caps lock) or G (with shift)
            (KeyCode::Char('G'), _) | (KeyCode::Char('g'), KeyModifiers::SHIFT) => self.bottom(),

            // shift + j/k = 5 skips
            (KeyCode::Char('J'), _) | (KeyCode::Char('j'), KeyModifiers::SHIFT) => self.next(5),
            (KeyCode::Char('K'), _) | (KeyCode::Char('k'), KeyModifiers::SHIFT) => self.prev(5),
            _ => {}
        }
    }

    pub fn handle_mouse(&mut self, mouse_event: &MouseEvent) {
        match mouse_event.kind {
            crossterm::event::MouseEventKind::ScrollUp => {
                self.prev(1);
            }

            crossterm::event::MouseEventKind::ScrollDown => {
                self.next(1);
            }
            _ => {}
        }
    }

    pub fn top(&mut self) {
        if self.data_len != 0 {
            self.select_index(0);
        }
    }

    pub fn bottom(&mut self) {
        if self.data_len != 0 {
            self.select_index(self.data_len - 1);
        }
    }

    pub fn next(&mut self, skip: usize) {
        if self.data_len != 0 {
            self.select_index((self.selected_index + skip) % self.data_len);
        }
    }

    pub fn prev(&mut self, skip: usize) {
        if self.data_len != 0 {
            if self.selected_index < skip {
                self.selected_index =
                    (self.data_len + self.selected_index - (skip % self.data_len)) % self.data_len;
            } else {
                self.selected_index -= skip;
            }

            self.select_index(self.selected_index); // this is kinda wrong but it works
        }
    }
}
