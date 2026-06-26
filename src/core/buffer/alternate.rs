use crate::core::buffer::TerminalBuffer;
use crate::core::cell::Cell;
use crate::core::grid::Grid;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct AlternateState {
    pub grid: Grid,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub current_style: crate::core::CellStyle,
    pub scrollback: VecDeque<Vec<Cell>>,
}

impl TerminalBuffer {
    pub fn is_alternate_screen(&self) -> bool {
        self.alternate_state.is_some()
    }

    pub fn enter_alternate_screen(&mut self) {
        if self.alternate_state.is_some() {
            return;
        }

        let state = AlternateState {
            grid: std::mem::take(&mut self.grid),
            cursor_x: self.cursor_x,
            cursor_y: self.cursor_y,
            current_style: self.current_style,
            scrollback: std::mem::take(&mut self.scrollback),
        };

        self.grid = Grid::new(state.grid.width(), state.grid.height());
        self.alternate_state = Some(Box::new(state));
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.current_style = crate::core::CellStyle::default();
    }

    pub fn exit_alternate_screen(&mut self) {
        if let Some(state) = self.alternate_state.take() {
            self.grid = state.grid;
            self.cursor_x = state.cursor_x;
            self.cursor_y = state.cursor_y;
            self.current_style = state.current_style;
            self.scrollback = state.scrollback;
        }
    }
}
