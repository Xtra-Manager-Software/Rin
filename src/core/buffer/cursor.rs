use crate::core::buffer::TerminalBuffer;
use crate::parser::CursorStyle;

impl TerminalBuffer {
    pub fn cursor_pos(&self) -> (usize, usize) {
        (self.cursor_x, self.cursor_y)
    }

    pub fn cursor_style(&self) -> CursorStyle {
        self.cursor_style
    }

    pub fn advance_to_next_tab_stop(&mut self) {
        let width = self.grid.width();
        for x in (self.cursor_x + 1)..width {
            if self.tab_stops.get(x).copied().unwrap_or(false) {
                self.cursor_x = x;
                return;
            }
        }
        self.cursor_x = width.saturating_sub(1);
    }
}
