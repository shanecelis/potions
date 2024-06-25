use std::cmp::Ordering;
use std::collections::BinaryHeap;
use approx::abs_diff_eq;
use derived_deref::{Deref, DerefMut};
// use color_art::Color;
use bevy_math::{IVec2, Vec2};

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct Color(color_art::Color);

impl From<color_art::Color> for Color {
    fn from(c: color_art::Color) -> Self {
        Self(c)
    }
}

///
///
#[derive(Debug, Clone)]
pub struct Vial {
    pub layers: Vec<Layer>,
    pub objects: Vec<Object>,
    pub max_volume: f64,
    pub glass: Color,
    pub size: Vec2,

}

impl Default for Vial {
    fn default() -> Self {
        Self {
            layers: vec![],
            max_volume: 100.0,
            objects: vec![],
            size: Vec2::new(5.0, 15.0),
            glass: color_art::Color::from_rgba(255, 255, 255, 0.5)
                .unwrap()
                .into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Layer {
    Liquid { id: usize, volume: f64 },
    // Empty,
}

#[derive(Debug, Clone)]
pub struct Object {
    pub kind: ObjectKind,
    pub pos: Vec2,
    pub size: usize,
}

#[derive(Deref)]
pub struct ByHeight<'a>(usize, #[target] &'a Object);

impl<'a> PartialOrd for ByHeight<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.pos.y.partial_cmp(&other.pos.y)
    }
}

impl<'a> Ord for ByHeight<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.pos.y.partial_cmp(&other.pos.y).or(abs_diff_eq!(self.pos.y, other.pos.y, epsilon = 0.1).then_some(Ordering::Equal)).unwrap_or(Ordering::Less)
    }
}

impl<'a> PartialEq for ByHeight<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.pos.y.eq(&other.pos.y)
    }
}

impl<'a> Eq for ByHeight<'a> { }

#[derive(Debug, Clone)]
pub enum ObjectKind {
    Seed,
    Creature,
    Plant,
}

// pub enum Liquid {
//     Water,
//     Oil,
// }

impl Layer {
    // pub fn color(&self) -> Color {
    //     match self {
    //         Layer::Liquid { id, .. } => {

    //         },
    //         Layer::Object(_) => todo!(),
    //         Layer::Empty => Color::from_rgb(0, 0, 0).unwrap()
    //     }
    // }
    pub fn volume(&self) -> f64 {
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
    Object
}

pub enum TransferError {
}

pub trait Lerp<T> {
    // type Data;
    fn lerp(&self, a: &T, b: &T, t: f64) -> Option<(Vial, Vial)>;
    fn result(&self, a: &T, b: &T) -> (Vial, Vial) {
        self.lerp(a, b, 1.0).unwrap()
    }
}
#[derive(Debug, Clone)]
pub struct LiquidTransfer;
#[derive(Debug, Clone)]
pub struct ObjectTransfer;

impl Lerp<Vial> for Transfer {
    fn lerp(&self, a: &Vial, b: &Vial, t: f64) -> Option<(Vial, Vial)> {
        if t > 1.0 {
            return None;
        }
        let mut a = a.clone();
        let mut b = b.clone();
        match self {
            Transfer::Liquid => {
                let top_layer_a = a.layers.len() - 1;
                let mut objects_top_a: Vec<usize> = a.objects.iter().enumerate().filter_map(|(i, o)|
                                                                            match a.in_layer(o.pos) {
                                                                                Some(VialLoc::Top) => Some(i),
                                                                                Some(VialLoc::Layer(l)) if l == top_layer_a => Some(i),
                                                                                _ => None,
                                                                            }).collect();
                let Layer::Liquid { volume: ref mut volume_a, id: id_a } = a.layers.last_mut().unwrap() else { panic!() };
                let total_volume_b = b.vol();
                if let Some(Layer::Liquid { volume: ref mut volume_b, id: id_b }) = b.layers.last_mut() {
                    assert_eq!(id_a, id_b);
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
                        *volume_a = 0.0;
                        *volume_b += *volume_a * t;
                    }
                }
                if matches!(a.layers.last().unwrap(), Layer::Liquid { volume: 0.0, .. }) {
                    a.layers.pop();
                }
                // We also have to transfer objects that are in the liquid.
                let transfer_count = (objects_top_a.len() as f64 * t).ceil() as usize;

                objects_top_a.sort_unstable();
                objects_top_a.reverse();
                for i in 0..transfer_count {
                    let mut obj = a.objects.swap_remove(objects_top_a[i]);
                    obj.pos.y = b.size.y * 1.1;
                    b.objects.push(obj);
                }
            }
            Transfer::Object => {
                if a.objects.len() <= 0 {
                    return None;
                }
                let mut heap: BinaryHeap<ByHeight> = a.objects.iter().enumerate().map(|(i, o)| ByHeight(i, o)).collect();
                let Some(ByHeight(top_index, top)) = heap.pop() else { panic!() };
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
                    obj.pos.y = b.size.y * 1.1;
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

enum VialLoc {
    Layer(usize),
    Top,
}

impl Vial {

    // pub fn objects_in_layer(&self, layer_index: usize) -> Vec<usize> { //impl Iterator<Item = usize> {
    //     let height_per_vol = self.size.y / self.max_volume;
    //     if layer_index == self.layers.len() {

    //     } else {

    //     }
    //     for (i, layer) in self.layers.iter().enumerate() {

    // }

    pub fn in_layer(&self, point: Vec2) -> Option<VialLoc> {
        let height_per_vol = self.size.y as f64 / self.max_volume;

        let mut height = 0.0;
        for (i, layer) in self.layers.iter().enumerate() {
            height += height_per_vol * layer.volume();
            if (point.y as f64) < height {
                return Some(VialLoc::Layer(i));
            }
        }
        if point.y < self.size.y {
            return Some(VialLoc::Top);
        }
        None
    }

    pub fn vol(&self) -> f64 {
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
        self.layers.last()
            .and_then(|a| match &a {
                &Layer::Liquid {
                    id: color_a,
                    volume: volume_a,
                } =>
                    other.layers.last()
                         .and_then(|b|
                                   match b {
                                       &Layer::Liquid {
                                           id: color_b,
                                           volume: volume_b,
                                       }
                                       => {
                                           if *color_a == color_b {
                                               let empty_volume_b = other.max_volume - other.vol();
                                               if empty_volume_b > 0.0 {
                                                   return Some(Transfer::Liquid)
                                               } else {
                                                   None
                                               }
                                           } else {
                                               None
                                           }
                                       }
                                       _ => None,
                                   }
                         ),
                _ => todo!()
            }).or((self.objects.len() > 0).then_some(Transfer::Object))
    }

    pub fn transition(&self) -> Option<Transition> {
        if self.layers.len() == 0 {
            let mut a = self.clone();
            let mut accum = vec![];
            for obj in a.objects.iter_mut().filter(|o| o.pos.y > a.size.y && o.size > 1) {
                let smaller_count = 3;
                obj.size = obj.size.saturating_sub(1);
                for i in 0..smaller_count {
                    accum.push(obj.clone());
                }
            }
            if accum.len() <= 0 {
                None
            } else {
                for o in accum {
                    a.objects.push(o);
                }
                Some(Transition::BreakSeed(a))
            }
        } else {
            None
        }
    }
}

