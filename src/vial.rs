use approx::abs_diff_eq;
use derived_deref::{Deref, DerefMut};
// use color_art::Color;

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
    Seed,
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

pub enum Transition {
    MoveDown(Vial),
    BreakSeed(Vial),
}

pub enum Transfer {
    Liquid,
    Object
}

pub enum TransferError {


}

// impl Iterator for Transfer {
//     type Item = (Vial, Vial);
//     fn next(&mut self) -> Option<Self::Item> {


//     }
// }

impl Transfer {
    fn result(&self, a: &Vial, b: &Vial) -> (Vial, Vial) {
        let mut a = a.clone();
        let mut b = b.clone();
        match self {
            Transfer::Liquid => {
                let Layer::Liquid { volume: ref mut volume_a, id: id_a } = a.layers.last_mut().unwrap() else { panic!() };
                if let Some(Layer::Liquid { volume: ref mut volume_b, id: id_b }) = b.layers.last_mut() {
                    assert_eq!(id_a, id_b);
                    let empty_volume_b = b.volume - b.vol();
                    assert!(empty_volume_b > 0.0);
                    let t = 1.0;
                    if *volume_a > empty_volume_b * t {
                        // We pour some.
                        *volume_a -= empty_volume_b * t;
                        a.discard_empties();


                        *volume_b += empty_volume_b * t;
                        b.discard_empties();
                    } else {
                        // We pour all.
                        a.layers.pop();
                        *volume_b += *volume_a * t;
                    }
                }            }
            Transfer::Object => {
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
            }
            _ => todo!(),
        };
        (a, b)
    }
}
//     fn has_lerp(&self) -> bool {
//         match self {
//             Transfer::Liquid(_, _) => true,
//             Transfer::Object(_, _) => true,
//         }
//     }

//     fn lerp(&self, t: f64) -> Option<(Vial, Vial)> {
//         match self {
//             Transfer::Liquid(a, b) => true,
//             Transfer::Object(_, _) => true,
//         }
//     }
// }

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
    pub fn pour(&self, other: &Vial, t: f64) -> Option<Transfer> {
        self.top_layer()
            .and_then(|a| match &a {
                &Layer::Object(o) => {
                    Some(Transfer::Object)
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
                                                   let mut s = self.clone();
                                                   let mut o = other.clone();
                                                   if *volume_a > empty_volume_b * t {
                                                       // We pour some.
                                                       *s.layers.last_mut().unwrap() = Layer::Liquid {
                                                           volume: volume_a - empty_volume_b * t,
                                                           id: *color_a,
                                                       };
                                                       s.discard_empties();

                                                       *o.layers.last_mut().unwrap() = Layer::Liquid {
                                                           volume: volume_b + empty_volume_b * t,
                                                           id: color_b,
                                                       };
                                                       o.discard_empties();
                                                   } else {
                                                       // We pour all.
                                                       s.layers.pop();

                                                       *o.layers.last_mut().unwrap() = Layer::Liquid {
                                                           volume: volume_b + volume_a,
                                                           id: color_b,
                                                       };
                                                   }
                                                   return Some(Transfer::Liquid(s, o))
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
