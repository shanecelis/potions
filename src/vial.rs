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

#[derive(Debug, Clone)]
pub struct Vial {
    pub layers: Vec<Layer>,
    pub volume: f64,
    pub glass: Color,
}

#[derive(Debug, Clone)]
pub enum Layer {
    Liquid { id: usize, volume: f64 },
    Object(Object),
    Empty,
}

#[derive(Debug, Clone)]
pub enum Object {
    Seed, //{ offset: Vec2 },
    BrokenSeed,
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
            Layer::Object(_) => todo!(),
            Layer::Empty => 0.0,
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
    Liquid(LiquidTransfer),
    Object(ObjectTransfer)
}

pub enum TransferError {
}

pub trait Lerp<T> {
    type Data;
    fn lerp(&self, a: &T, b: &T, t: f64) -> Option<(Vial, Vial, Self::Data)>;
    fn result(&self, a: &T, b: &T) -> (Vial, Vial, Self::Data) {
        self.lerp(a, b, 1.0).unwrap()
    }
}
#[derive(Debug, Clone)]
pub struct LiquidTransfer;
#[derive(Debug, Clone)]
pub struct ObjectTransfer;

impl Lerp<Vial> for LiquidTransfer {
    type Data = ();
    fn lerp(&self, a: &Vial, b: &Vial, t: f64) -> Option<(Vial, Vial, ())> {
        let mut a = a.clone();
        let mut b = b.clone();
        let Layer::Liquid { volume: ref mut volume_a, id: id_a } = a.layers.last_mut().unwrap() else { panic!() };
        let total_volume_b = b.vol();
        if let Some(Layer::Liquid { volume: ref mut volume_b, id: id_b }) = b.layers.last_mut() {
            assert_eq!(id_a, id_b);
            let empty_volume_b = b.volume - total_volume_b;
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
        Some((a, b, ()))
    }
}


impl Lerp<Vial> for ObjectTransfer {
    type Data = f64;
    fn lerp(&self, a: &Vial, b: &Vial, t: f64) -> Option<(Vial, Vial, f64)> {
        if t > 1.0 {
            None
        } else {
            let mut a = a.clone();
            let mut b = b.clone();
            let Layer::Object(o) = a.layers.last_mut().unwrap() else { panic!(); };
            match o {
                Object::Seed if b.layers.len() == 0 => {
                    a.layers.pop();
                    b.layers.push(Layer::Object(Object::BrokenSeed));
                }
                _ => {
                    b.layers.push(a.layers.pop().unwrap())
                }
            }
            Some((a, b, t))
        }
    }
}

impl Vial {
    pub fn top_layer(&self) -> Option<&Layer> {
        self.layers.last()
    }

    pub fn vol(&self) -> f64 {
        self.layers.iter().map(|l: &Layer| l.volume()).sum()
    }

    pub fn discard_empties(&mut self) {
        if let Some(vol) = self.top_layer().map(|l| l.volume()) {
            if abs_diff_eq!(vol, 0.0, epsilon = 0.01) {
                self.layers.pop();
            }
        }
    }

    /// Pour self into other potion.
    pub fn pour(&self, other: &Vial) -> Option<Transfer> {
        self.top_layer()
            .and_then(|a| match &a {
                &Layer::Object(o) => {
                    Some(Transfer::Object(ObjectTransfer))
                },
                &Layer::Liquid {
                    id: color_a,
                    volume: volume_a,
                } =>
                    other.top_layer()
                         .and_then(|b|
                                   match b {
                                       &Layer::Liquid {
                                           id: color_b,
                                           volume: volume_b,
                                       }
                                       => {
                                           if *color_a == color_b {
                                               let empty_volume_b = other.volume - other.vol();
                                               if empty_volume_b > 0.0 {
                                                   return Some(Transfer::Liquid(LiquidTransfer))
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
            })
    }

    pub fn transition(&self) -> Option<Transition> {
        if self.layers.len() == 1 {
            if matches!(self.layers[0], Layer::Object(Object::Seed)) {
                let mut s = self.clone();
                s.layers[0] = Layer::Object(Object::BrokenSeed);
                return Some(Transition::BreakSeed(s));
            }
        }
        None
    }
}

impl Default for Vial {
    fn default() -> Self {
        Self {
            layers: vec![],
            volume: 100.0,
            glass: color_art::Color::from_rgba(255, 255, 255, 0.5)
                .unwrap()
                .into(),
        }
    }
}
