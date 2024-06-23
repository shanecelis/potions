use derived_deref::{Deref, DerefMut};
use approx::abs_diff_eq;
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
    pub fn pour(&self, other: &Vial, t: f64) -> Option<(Vial, Vial)> {
        self.top_layer().and_then(|a| {
            other.top_layer().and_then(|b| {
                match (a, b) {
                    (
                        &Layer::Liquid {
                            id: color_a,
                            volume: volume_a,
                        },
                        &Layer::Liquid {
                            id: color_b,
                            volume: volume_b,
                        },
                    ) => {
                        if color_a == color_b {
                            let empty_volume_b = other.volume - other.vol();
                            if empty_volume_b > 0.0 {
                                let mut s = self.clone();
                                let mut o = other.clone();
                                if volume_a > empty_volume_b * t {
                                    // We pour some.
                                    *s.layers.last_mut().unwrap() = Layer::Liquid {
                                        volume: volume_a - empty_volume_b * t,
                                        id: color_a,
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
                                Some((s, o))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            })
        })
    }
}

impl Default for Vial {
    fn default() -> Self {
        Self {
            layers: vec![],
            volume: 100.0,
            glass: color_art::Color::from_rgba(255, 255, 255, 0.5).unwrap().into(),
        }
    }
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
    BrokenSeeds,
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

