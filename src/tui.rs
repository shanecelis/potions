use super::{Layer, Vial};
use ratatui::{
    prelude::*,
    // widgets::{canvas::*, *},
};
// use std::iter;


fn to_color(color: color_art::Color) -> Color {
    Color::Rgb(color.red(), color.green(), color.blue())
}

impl Widget for Vial {
    #[allow(clippy::cast_possible_truncation)]
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border = Style::new().bg(to_color(self.glass));
        // let view_volume = (area.width - 2) * (area.height - 1);
        let volume_per_row = self.volume / (area.height - 1) as f64;

        if area.height > 1 {
            // Draw bottom.
            buf.set_string(
                area.x + 1,
                area.y + area.height - 1,
                " ".repeat((area.width - 2) as usize),
                border,
            );
        }
        for j in area.y..area.y + area.height {
            // Draw vertical sides.
            buf.set_string(area.x, j, " ", border);
            buf.set_string(area.x + area.width - 1, j, " ", border);
        }
        // Draw contents.
        let mut j = area.y + area.height - 2;
        // let mut volume_remaining = self.volume;
        let mut slop = 0.0;
        // dbg!(area);
        for layer in self.layers {
            match layer {
                Layer::Liquid { color, mut volume } => {
                    let style = Style::new().bg(to_color(color));
                    volume += slop;

                    while volume > 0.0 {
                        buf.set_string(area.x + 1, j, " ".repeat((area.width - 2) as usize), style);
                        volume -= volume_per_row;
                        j = j.saturating_sub(1);
                    }
                    slop = volume;
                    // dbg!(volume);
                    // dbg!(slop);
                }
                _ => todo!(),
            }
        }
    }
}
