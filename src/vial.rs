use approx::abs_diff_eq;
use derived_deref::{Deref, DerefMut};
use std::collections::BinaryHeap;
use crate::Palette;
use bevy_color::{Mix, Srgba};
use bevy_math::Vec2;
use serde::{Deserialize, Serialize};
use super::{Object, ObjectFlags, ByHeight};
use crate::user_data::{UserData};

#[derive(Debug, Clone, Deref, DerefMut, Deserialize, Serialize)]
pub struct Color(color_art::Color);

impl From<color_art::Color> for Color {
    fn from(c: color_art::Color) -> Self {
        Self(c)
    }
}

impl From<Color> for bevy_color::Srgba {
    fn from(c: Color) -> Self {
        fn f(f: u8) -> f32 {
            f as f32 / 255.0
        }
        Srgba {
            red: f(c.red()),
            green: f(c.green()),
            blue: f(c.blue()),
            alpha: 1.0,
        }
    }
}

impl From<bevy_color::Srgba> for Color {
    fn from(c: bevy_color::Srgba) -> Self {
        let bevy_color::Srgba {
            red, green, blue, ..
        } = c;
        fn f(f: f32) -> u8 {
            (f * 255.0) as u8
        }
        Self(color_art::Color::new(f(red), f(green), f(blue), 1.0))
    }
}

// #[quantity(Length * Length)]
// #[ref_unit(Square_Meter, "m²", NONE, "Reference ]
// pub struct Volume;

/// A vial holds liquids and objects in it.
///
/// Here's what the coordinate space looks like.
///
///              ^
///              |
/// y in [0, h]  +-------------+
///              |             |
///              |             |
///              |             |
///              |             |
///              |             |
///              |             |
///              +-------------+
///              |             |
///              |   layer1    |
///              |             |
///              +-------------+
///              |             |
///              |   layer0    |
///              |             |
///              +-------------+->   x in [0, w]
///
///                    Vial
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Vial {
    pub layers: Vec<Layer>,
    pub objects: Vec<Object>,
    pub max_volume: f32,
    pub glass: Color,
    /// units: mm
    pub size: Vec2,
}

impl Default for Vial {
    fn default() -> Self {
        Self {
            layers: vec![],
            max_volume: 100.0,
            objects: vec![],
            size: Vec2::new(25.0, 75.0),
            glass: color_art::Color::from_rgba(255, 255, 255, 0.5)
                .unwrap()
                .into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Layer {
    Liquid { id: usize, volume: f32 },
    // Empty,
}

// #[derive(Debug, Clone)]
// pub struct LiquidProp {
//     pub density: f32,
// }
//


// pub enum Liquid {
//     Water,
//     Oil,
// }

impl Layer {
    pub fn volume(&self) -> f32 {
        match self {
            Layer::Liquid { volume, .. } => *volume,
            // Layer::Empty => 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Transition {
    MoveDown(Vial),
    BreakSeed(Vial),
}

#[derive(Debug, Clone)]
pub enum Transfer {
    Liquid,
    Object,
}

// pub enum TransferError {}

pub trait Lerp<T> {
    fn lerp(&self, a: &T, b: &T, t: f32) -> Option<(Vial, Vial)>;
    fn result(&self, a: &T, b: &T) -> (Vial, Vial) {
        self.lerp(a, b, 1.0).unwrap()
    }
}

impl Lerp<Vial> for Transfer {
    fn lerp(&self, a: &Vial, b: &Vial, t: f32) -> Option<(Vial, Vial)> {
        if t > 1.0 {
            return None;
        }
        let mut a = a.clone();
        let mut b = b.clone();
        match self {
            Transfer::Liquid => {
                let mut objects_top_a: Vec<usize> = if !a.layers.is_empty() {
                    let top_layer_a = a.layers.len() - 1;
                    a.objects
                     .iter()
                     .enumerate()
                     .filter_map(|(i, o)| match a.in_layer(o.pos, o.size) {
                         Some(VialLoc::Top { .. }) => Some(i),
                         Some(VialLoc::Layer { index: l, .. }) if l == top_layer_a => Some(i),
                         _ => None,
                     })
                     .collect()
                } else {
                    return None;
                    // let r = 0..a.objects.len();
                    // r.collect()
                };
                let Layer::Liquid {
                    volume: ref mut volume_a,
                    id: id_a,
                } = a.layers.last_mut().unwrap();
                let total_volume_b = b.vol();
                if b.layers.is_empty() {
                    b.layers.push(Layer::Liquid {
                        volume: 0.0,
                        id: *id_a,
                    });
                }
                if let Some(Layer::Liquid {
                    volume: ref mut volume_b,
                    id: id_b,
                }) = b.layers.last_mut()
                {
                    if id_a != id_b {
                        return None;
                    }
                    // assert_eq!(id_a, id_b);
                    let empty_volume_b = b.max_volume - total_volume_b;
                    assert!(empty_volume_b > 0.0);
                    // let t = 1.0;
                    if *volume_a > empty_volume_b * t {
                        // We pour some.
                        *volume_a -= empty_volume_b * t;
                        a.discard_empties();
                        *volume_b += empty_volume_b * t;
                        b.discard_empties();
                    } else {
                        // We pour all.
                        *volume_b += *volume_a * t;
                        *volume_a = 0.0;
                    }
                }
                if matches!(a.layers.last().unwrap(), Layer::Liquid { volume: 0.0, .. }) {
                    a.layers.pop();
                }
                // We also have to transfer objects that are in the liquid.
                let transfer_count = (objects_top_a.len() as f32 * t).ceil() as usize;

                objects_top_a.sort_unstable();
                objects_top_a.reverse();
                // for i in 0..transfer_count {
                for i in objects_top_a.into_iter().take(transfer_count) {
                    let mut obj = a.objects.swap_remove(i);
                    // XXX: This is causing a panic.
                    obj.pos.y = b.size.y;
                    // obj.pos.y = b.size.y * 0.8;
                    b.objects.push(obj);
                }
            }
            Transfer::Object => {
                if a.objects.is_empty() {
                    return None;
                }
                let mut heap: BinaryHeap<ByHeight> = a
                    .objects
                    .iter()
                    .enumerate()
                    .map(|(i, o)| ByHeight(i, o))
                    .collect();
                let ByHeight(top_index, top) = heap.pop().expect("object");
                let mut transfers = vec![];
                transfers.push(top_index);
                while let Some(ByHeight(i, obj)) = heap.pop() {
                    if abs_diff_eq!(top.pos.y, obj.pos.y, epsilon = 0.1) {
                        transfers.push(i);
                    }
                }
                transfers.sort();
                transfers.reverse();
                for i in transfers {
                    let mut obj = a.objects.swap_remove(i);
                    // XXX: This is causing a panic.
                    obj.pos.y = b.size.y;
                    // obj.pos.y = b.size.y * 0.8;
                    //
                    obj.flags |= ObjectFlags::ENTER_VIAL;
                    if b.layers.is_empty() && obj.size > 1.0 {
                        obj.flags |= ObjectFlags::EXPECT_BREAK;
                    }
                    b.objects.push(obj);
                }

                // let Layer::Object { obj: o, .. } = a.layers.last_mut().unwrap() else { panic!(); };
                // match o {
                //     Object::Seed if b.layers.len() == 0 => {
                //         a.layers.pop();
                //         b.layers.push(Layer::Object { obj: Object::BrokenSeed, pos: None });
                //     }
                //     _ => {
                //         b.layers.push(a.layers.pop().unwrap())
                //     }
                // }
            }
        }
        Some((a, b))
    }
}

pub enum VialLoc {
    Layer { index: usize, height: f32 },
    Top { height: f32 },
}

impl Vial {
    // pub fn objects_in_layer(&self, layer_index: usize) -> Vec<usize> { //impl Iterator<Item = usize> {
    //     let height_per_vol = self.size.y / self.max_volume;
    //     if layer_index == self.layers.len() {

    //     } else {

    //     }
    //     for (i, layer) in self.layers.iter().enumerate() {

    // }

    pub fn in_layer(&self, point: Vec2, r: f32) -> Option<VialLoc> {
        let height_per_vol = self.size.y / self.max_volume;

        let mut height = 0.0;
        for (i, layer) in self.layers.iter().enumerate() {
            height += height_per_vol * layer.volume();
            if point.y < height + r {
                return Some(VialLoc::Layer { index: i, height });
            }
        }
        if point.y < self.size.y + r {
            return Some(VialLoc::Top {
                height: self.size.y,
            });
        }
        None
    }

    pub fn vol(&self) -> f32 {
        self.layers.iter().map(|l: &Layer| l.volume()).sum()
    }

    pub fn discard_empties(&mut self) {
        if let Some(vol) = self.layers.last().map(|l| l.volume()) {
            if abs_diff_eq!(vol, 0.0, epsilon = 0.01) {
                self.layers.pop();
            }
        }
    }

    /// Pour self into other potion.
    pub fn pour(&self, other: &Vial) -> Option<Transfer> {
        self.layers
            .last()
            .and_then(|a| match &a {
                &Layer::Liquid {
                    id: color_a,
                    ..
                    // volume: volume_a,
                } => other.layers.last().and_then(|b| match b {
                    &Layer::Liquid {
                        id: color_b,
                        ..
                        // volume: volume_b,
                    } => {
                        if *color_a == color_b {
                            let empty_volume_b = other.max_volume - other.vol();
                            if empty_volume_b > 0.0 {
                                Some(Transfer::Liquid)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    // _ => None,
                }).or((other.layers.is_empty()).then_some(Transfer::Liquid)),
                // _ => todo!(),
            })
            .or((!self.objects.is_empty()).then_some(Transfer::Object))
    }

    pub fn transition(&self) -> Option<Transition> {
        if !self.objects.iter().any(|o| o.flags.contains(ObjectFlags::BREAK)) {
            None
        } else {
            let mut a = self.clone();
            let mut additions = vec![];

            let mut next_index = a.objects.len();
            for obj in &mut a.objects {
                if obj.flags.contains(ObjectFlags::BREAK) {
                    obj.flags.remove(ObjectFlags::BREAK);
                    obj.size /= 2.0;
                    let mut o = obj.clone();
                    o.id = UserData::object(next_index as u8).into();
                    next_index += 1;
                    additions.push(o);
                }
            }
            a.objects.extend(additions);
            Some(Transition::BreakSeed(a))
        // if self.layers.len() == 0 {
        //     let mut a = self.clone();
        //     let mut accum = vec![];
        //     for obj in a
        //         .objects
        //         .iter_mut()
        //         .filter(|o| o.pos.y > a.size.y && o.size > 1.0)
        //     {
        //         let divide_into_count = 2;
        //         obj.size = obj.size.saturating_sub(1.0);
        //         for i in 0..smaller_count {
        //             // XXX: Change where they are
        //             accum.push(obj.clone());
        //         }
        //     }
        //     if accum.len() <= 0 {
        //         None
        //     } else {
        //         for o in accum {
        //             a.objects.push(o);
        //         }
        //         Some(Transition::BreakSeed(a))
        //     }
        // } else {
        //     None
        // }
        }
    }

    pub fn mix(&mut self, palette: &mut Palette) -> bool {
        if self.layers.len() < 2 {
            false
        } else {
            let Layer::Liquid {
                id: top_id,
                volume: top_volume,
            } = self.layers.pop().unwrap();
            let Layer::Liquid {
                id: bottom_id,
                volume: bottom_volume,
            } = self.layers.pop().unwrap();
            let top_color: Srgba = palette[top_id].clone().into();
            let bottom_color: Srgba = palette[bottom_id].clone().into();
            // let color = (top_volume * top_color + bottom_volume * bottom_color) / (top_volume + bottom_volume);
            let p = bottom_volume / (top_volume + bottom_volume);
            let color: Srgba = top_color.mix(&bottom_color, p);
            let new_id = palette.len();
            palette.push(color.into());
            let mix = Layer::Liquid {
                volume: top_volume + bottom_volume,
                id: new_id,
            };
            self.layers.push(mix);
            true
        }
    }
}

#[cfg(test)]
mod test {
    use quantities::prelude::*;
    use super::*;

    #[test]
    fn test_quantities() {
        use quantities::length::*;
        let a = Amnt!(1.0) * METER;
        let b = Amnt!(1.0) * CENTIMETER;
        let c = Amnt!(1.0) * MILLIMETER;
        assert_eq!(a + b, Amnt!(1.01) * METER);
        assert_eq!(a + c, Amnt!(1.001) * METER);
        assert_eq!(a.to_string(), "1 m");
    }

    #[test]
    fn test_pour() {
        let a = Vial {
                    layers: vec![Layer::Liquid {
                        id: 0,
                        volume: 50.0,
                    }],
                    ..Default::default()
                };
        let b = Vial::default();
        assert_eq!(a.vol(), 50.0);
        assert_eq!(b.vol(), 0.0);
        if let Some(transfer) = a.pour(&b) {
            assert!(matches!(transfer, Transfer::Liquid));
            if let Some((a, b)) = transfer.lerp(&a, &b, 1.0) {
                assert_eq!(a.vol(), 0.0);
                assert_eq!(b.vol(), 50.0);
            } else {
                panic!();
            }
        } else {
            panic!();
        };

    }
}
