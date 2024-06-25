use super::{Layer, Palette, Vial, Object};
use ratatui::prelude::*;

impl From<crate::Color> for Color {
    fn from(color: crate::Color) -> Self {
        Color::Rgb(color.red(), color.green(), color.blue())
    }
}

#[derive(Debug, Clone)]
pub struct VialWidget<'a>(pub &'a Vial, pub &'a Palette);

impl<'a> Widget for VialWidget<'a> {
    #[allow(clippy::cast_possible_truncation)]
    fn render(self, area: Rect, buf: &mut Buffer) {
        let VialWidget(vial, palette) = self;
        let border = Style::new().bg(vial.glass.clone().into());
        // let view_volume = (area.width - 2) * (area.height - 1);
        let volume_per_row = vial.volume / (area.height - 1) as f64;

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
        // let mut volume_remaining = vial.volume;
        let mut slop = 0.0;
        // dbg!(area);
        for layer in &vial.layers {
            match layer {
                Layer::Liquid { id, mut volume } => {
                    let style = Style::new().bg(palette[*id].clone().into());
                    volume += slop;

                    while volume > 0.0 {
                        buf.set_string(area.x + 1, j, " ".repeat((area.width - 2) as usize), style);
                        volume -= volume_per_row;
                        j = j.saturating_sub(1);
                    }
                    slop = volume;
                    // dbg!(volume);
                    // dbg!(slop);
                },
                Layer::Object { obj: o, pos } => match o {
                    Object::Seed => {
                        let size = 4;
                        for j in 0..size/2 {
                            buf.set_string(
                                area.x + area.width / 2 - size / 2,
                                area.y + area.height - 2 - j,
                                " ".repeat(size as usize),
                                border,
                            );
                        }
                    },
                    _ => todo!(),
                }
                _ => todo!(),
            }
        }
    }
}
