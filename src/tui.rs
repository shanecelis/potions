use super::{Layer, ObjectKind, Palette, Vial};
use bevy_math::Vec2;
use ratatui::prelude::*;

impl From<crate::Color> for Color {
    fn from(color: crate::Color) -> Self {
        Color::Rgb(color.red(), color.green(), color.blue())
    }
}

#[derive(Debug, Clone)]
pub struct VialWidget<'a>(pub &'a Vial, pub &'a Palette);

fn find_margins(size: Vec2, area: Vec2) -> Vec2 {
    let scale = f32::min(area.x / size.x, area.y / size.y);

    let new_size = scale * size;
    (area - new_size) / 2.0
}

impl<'a> Widget for VialWidget<'a> {
    #[allow(clippy::cast_possible_truncation)]
    fn render(self, area: Rect, buf: &mut Buffer) {
        let VialWidget(vial, palette) = self;
        // XXX: Render it with the correct aspect ratio.
        let margins = find_margins(
            Vec2::new(vial.size.x, vial.size.y / 2.0),
            Vec2::new(area.width as f32, area.height as f32),
        );
        // dbg!(margins);
        let [area] = Layout::vertical([Constraint::Percentage(100)])
            .horizontal_margin(margins.x as u16)
            .vertical_margin(margins.y as u16)
            .areas(area);
        // dbg!(area);
        let border = Style::new().bg(vial.glass.clone().into());
        // let view_volume = (area.width - 2) * (area.height - 1);
        let volume_per_row = vial.max_volume / (area.height - 1) as f32;

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
                }
            }
        }
        for object in &vial.objects {
            match object.kind {
                ObjectKind::Seed => {
                    let size = object.size as u16;
                    let x = (object.pos.x / self.0.size.x * (area.width - 3) as f32) as i16;
                    let y = (object.pos.y / self.0.size.y * (area.height - 2) as f32) as i16;
                    // dbg!((x, y));
                    for j in 0..(size / 2).max(1) {
                        buf.set_string(
                            ((area.x + (size / 2).max(1)) as i16 + x) as u16,
                            ((area.y + area.height - 2 - j) as i16 - y) as u16,
                            " ".repeat(size as usize),
                            border,
                        );
                    }
                }
                _ => todo!(),
            }
        }
    }
}
