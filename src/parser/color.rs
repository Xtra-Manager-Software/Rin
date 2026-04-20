// moved from: src/parser/ansi.rs
// path updated

use crate::core::Color;

pub fn ansi_color(n: u16) -> Color {
    match n {
        0 => Color::new(0, 0, 0),
        1 => Color::new(205, 49, 49),
        2 => Color::new(13, 188, 121),
        3 => Color::new(229, 229, 16),
        4 => Color::new(36, 114, 200),
        5 => Color::new(188, 63, 188),
        6 => Color::new(17, 168, 205),
        7 => Color::new(229, 229, 229),
        _ => Color::WHITE,
    }
}

pub fn ansi_bright_color(n: u16) -> Color {
    match n {
        0 => Color::new(102, 102, 102),
        1 => Color::new(241, 76, 76),
        2 => Color::new(35, 209, 139),
        3 => Color::new(245, 245, 67),
        4 => Color::new(59, 142, 234),
        5 => Color::new(214, 112, 214),
        6 => Color::new(41, 184, 219),
        7 => Color::new(255, 255, 255),
        _ => Color::WHITE,
    }
}

pub fn color_256(n: u8) -> Color {
    match n {
        0..=15 => {
            if n < 8 {
                ansi_color(n as u16)
            } else {
                ansi_bright_color((n - 8) as u16)
            }
        }
        16..=231 => {
            let n = n - 16;
            let r = (n / 36) % 6;
            let g = (n / 6) % 6;
            let b = n % 6;
            let to_rgb = |v: u8| if v == 0 { 0 } else { 55 + v * 40 };
            Color::new(to_rgb(r), to_rgb(g), to_rgb(b))
        }
        232..=255 => {
            let gray = 8 + (n - 232) * 10;
            Color::new(gray, gray, gray)
        }
    }
}
