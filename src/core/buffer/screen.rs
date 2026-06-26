use crate::core::cell::Cell;
use crate::core::buffer::TerminalBuffer;

impl TerminalBuffer {
    pub fn scrollback_len(&self) -> usize {
        self.scrollback.len()
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn scroll_by(&mut self, delta: i32) {
        let new_offset = (self.scroll_offset as i32 + delta)
            .max(0)
            .min(self.scrollback.len() as i32) as usize;
        self.scroll_offset = new_offset;
    }

    pub fn scroll_to(&mut self, offset: usize) {
        self.scroll_offset = offset.min(self.scrollback.len());
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scrollback_row(&self, index: usize) -> Option<&[Cell]> {
        self.scrollback.get(index).map(|v| v.as_slice())
    }

    pub fn set_scrollback_limit(&mut self, limit: usize) {
        self.scrollback_limit = limit;
        while self.scrollback.len() > limit {
            self.scrollback.pop_front();
        }
    }

    pub fn effective_scroll_region(&self) -> (usize, usize) {
        let height = self.grid.height();
        self.scroll_region
            .map(|(t, b)| (t, b.min(height.saturating_sub(1))))
            .unwrap_or((0, height.saturating_sub(1)))
    }

    pub fn scroll_up(&mut self, n: usize) {
        let width = self.grid.width();
        let (top, bottom) = self.effective_scroll_region();
        let full_height = self.grid.height().saturating_sub(1);

        if top == 0 && bottom == full_height {
            for y in 0..n.min(bottom + 1) {
                self.scrollback.push_back(self.grid.take_row(y));
            }
            while self.scrollback.len() > self.scrollback_limit {
                self.scrollback.pop_front();
            }
        }

        for y in (top + n)..=bottom {
            for x in 0..width {
                self.grid.swap_cells(x, y, x, y - n);
            }
        }

        let clear_start = if bottom + 1 >= n { bottom + 1 - n } else { top };
        for y in clear_start..=bottom {
            for x in 0..width {
                let _ = self.grid.set(x, y, Cell::default());
            }
        }

        if top == 0 && bottom == full_height {
            self.cursor_y = self.cursor_y.saturating_sub(n);
        }
    }

    pub fn scroll_down(&mut self, n: usize) {
        let width = self.grid.width();
        let (top, bottom) = self.effective_scroll_region();

        for y in (top..=(bottom.saturating_sub(n))).rev() {
            for x in 0..width {
                self.grid.swap_cells(x, y, x, y + n);
            }
        }
        for y in top..=(top + n - 1).min(bottom) {
            for x in 0..width {
                let _ = self.grid.set(x, y, Cell::default());
            }
        }
    }

    pub fn insert_lines_at_cursor(&mut self, n: usize) {
        let width = self.grid.width();
        let (_, bottom) = self.effective_scroll_region();
        let start = self.cursor_y;

        if start > bottom {
            return;
        }

        for y in (start..=(bottom.saturating_sub(n))).rev() {
            for x in 0..width {
                self.grid.swap_cells(x, y, x, y + n);
            }
        }

        for y in start..(start + n).min(bottom + 1) {
            for x in 0..width {
                let _ = self.grid.set(x, y, Cell::default());
            }
        }
    }

    pub fn delete_lines_at_cursor(&mut self, n: usize) {
        let width = self.grid.width();
        let (_, bottom) = self.effective_scroll_region();
        let start = self.cursor_y;

        if start > bottom {
            return;
        }

        for y in (start + n)..=bottom {
            for x in 0..width {
                self.grid.swap_cells(x, y, x, y - n);
            }
        }

        let clear_start = if bottom + 1 >= n {
            bottom + 1 - n
        } else {
            start
        };
        for y in clear_start..=bottom {
            for x in 0..width {
                let _ = self.grid.set(x, y, Cell::default());
            }
        }
    }

    pub fn erase_in_display(&mut self, mode: u8) {
        let width = self.grid.width();
        let height = self.grid.height();
        match mode {
            0 => {
                for x in self.cursor_x..width {
                    let _ = self.grid.set(x, self.cursor_y, Cell::default());
                }
                for y in (self.cursor_y + 1)..height {
                    for x in 0..width {
                        let _ = self.grid.set(x, y, Cell::default());
                    }
                }
            }
            1 => {
                for y in 0..self.cursor_y {
                    for x in 0..width {
                        let _ = self.grid.set(x, y, Cell::default());
                    }
                }
                for x in 0..=self.cursor_x.min(width.saturating_sub(1)) {
                    let _ = self.grid.set(x, self.cursor_y, Cell::default());
                }
            }
            _ => {}
        }
    }

    pub fn erase_in_line(&mut self, mode: u8) {
        let width = self.grid.width();
        match mode {
            0 => {
                for x in self.cursor_x..width {
                    let _ = self.grid.set(x, self.cursor_y, Cell::default());
                }
            }
            1 => {
                for x in 0..=self.cursor_x.min(width.saturating_sub(1)) {
                    let _ = self.grid.set(x, self.cursor_y, Cell::default());
                }
            }
            2 => {
                for x in 0..width {
                    let _ = self.grid.set(x, self.cursor_y, Cell::default());
                }
            }
            _ => {}
        }
    }

    pub fn erase_chars(&mut self, n: usize) {
        for i in 0..n {
            if self.cursor_x + i < self.grid.width() {
                let _ = self
                    .grid
                    .set(self.cursor_x + i, self.cursor_y, Cell::default());
            }
        }
    }
}
