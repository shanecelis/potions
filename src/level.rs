use super::*;
use derived_deref::{Deref, DerefMut};
use kolorwheel::{HslColor, KolorWheel, RgbColor, SpinMode};
use std::collections::BinaryHeap;

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct Palette(Vec<Color>);

pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    color_art::Color::from_rgb(r, g, b).unwrap().into()
}

impl Palette {
    pub fn new<I, J>(iter: I) -> Self
    where
        I: IntoIterator<Item = J>,
        J: Into<Color>,
    {
        Self(iter.into_iter().map(|x| x.into()).collect())
    }

    pub fn color(&self, layer: &Layer) -> Color {
        match layer {
            Layer::Object(_) => rgb(255, 255, 255),
            Layer::Liquid { id, .. } => self.0[*id].clone(),
            _ => todo!()
        }
    }
}

pub trait Level {
    fn palette(&self) -> &Palette;
    fn potions(&self) -> &[Vial];
    fn is_complete(&self, potions: &[Vial]) -> bool;
}

pub struct UnmixLevel {
    palette: Palette,
    potions: Vec<Vial>,
    goal: Goal,
}

impl Default for UnmixLevel {
    fn default() -> Self {
        Self {
            palette: Palette::new(vec![
                rgb(255, 0, 0),
                rgb(0, 255, 0),
                rgb(0, 0, 255),
            ]),
            potions: vec![],
            goal: Goal::Unmix,
        }
    }
}

impl UnmixLevel {
    fn new(vials: Vec<Vial>) -> Self {
        let heap: BinaryHeap<usize> = vials
            .iter()
            .flat_map(|v| &v.layers)
            .filter_map(|l| {
                if let Layer::Liquid { id, .. } = l {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();
        // let mut kw = KolorWheel::new(RgbColor { r: 255, g: 0, b: 0 }, heap.len());
        // let mut kw = KolorWheel::new(HslColor { h: 185.0, s: 70.0, l: 65.0 }, heap.len());
        let mut kw = KolorWheel::new(
            HslColor {
                h: 240.0,
                s: 82.0,
                l: 64.0,
            },
            heap.len(),
        );
        kw.with_hue(SpinMode::RelativeExcl(-360));
        let colors: Vec<color_art::Color> = kw
            .map(|c| RgbColor::from(c))
            .map(|RgbColor { r, g, b }| color_art::Color::from_rgb(r, g, b).unwrap())
            .collect();
        assert_eq!(colors.len(), heap.len());
        Self {
            palette: Palette::new(colors),
            potions: vials,
            goal: Goal::Unmix,
        }
    }
}

pub enum Goal {
    Unmix,
    BreakSeed,
}

// #[derive(Debug, Clone)]
// pub struct UnmixGoal;

// trait Goal {
//     fn is_complete(&self, potions: &[Vial]) -> bool;
// }

impl Goal {
    fn is_complete(&self, potions: &[Vial]) -> bool {
        match self {
            Goal::Unmix => potions.iter().all(|p| p.layers.len() <= 1),
            Goal::BreakSeed => potions.iter().any(|p| p.layers.iter().any(|l| matches!(l, Layer::Object(Object::BrokenSeed)))),
        }
    }
}

impl Level for UnmixLevel {
    fn palette(&self) -> &Palette {
        &self.palette
    }

    fn potions(&self) -> &[Vial] {
        &self.potions
    }

    fn is_complete(&self, potions: &[Vial]) -> bool {
        self.goal.is_complete(potions)
    }
}

pub fn levels() -> Vec<Box<dyn Level>> {
    vec![
        Box::new(UnmixLevel {
            goal: Goal::BreakSeed,
            potions: vec![
            Vial {
                layers: vec![
                    Layer::Object(Object::Seed),
                ],
                ..Default::default()
            },
            Vial {
                layers: vec![],
                ..Default::default()
            },
                ],
                ..Default::default()
             }),
        Box::new(UnmixLevel::new(vec![
            Vial {
                layers: vec![
                    Layer::Liquid {
                        id: 0,
                        volume: 50.0,
                    },
                    Layer::Liquid {
                        id: 1,
                        volume: 50.0,
                    },
                ],
                ..Default::default()
            },
            Vial {
                layers: vec![Layer::Liquid {
                    id: 1,
                    volume: 50.0,
                }],
                ..Default::default()
            },
        ])),
        Box::new(UnmixLevel::new(vec![
            Vial {
                layers: vec![
                    Layer::Liquid {
                        id: 0,
                        volume: 50.0,
                    },
                    Layer::Liquid {
                        id: 1,
                        volume: 50.0,
                    },
                ],
                ..Default::default()
            },
            Vial {
                layers: vec![
                    Layer::Liquid {
                        id: 1,
                        volume: 50.0,
                    },
                    Layer::Liquid {
                        id: 2,
                        volume: 25.0,
                    },
                ],
                ..Default::default()
            },
            Vial {
                layers: vec![Layer::Liquid {
                    id: 2,
                    volume: 50.0,
                }],
                ..Default::default()
            },
        ])),
        Box::new(UnmixLevel::new(vec![
            Vial {
                layers: vec![
                    Layer::Liquid {
                        id: 0,
                        volume: 50.0,
                    },
                    Layer::Liquid {
                        id: 1,
                        volume: 25.0,
                    },
                    Layer::Liquid {
                        id: 2,
                        volume: 25.0,
                    },
                ],
                ..Default::default()
            },
            Vial {
                layers: vec![
                    Layer::Liquid {
                        id: 1,
                        volume: 50.0,
                    },
                    Layer::Liquid {
                        id: 2,
                        volume: 25.0,
                    },
                ],
                ..Default::default()
            },
            Vial {
                layers: vec![Layer::Liquid {
                    id: 2,
                    volume: 50.0,
                }],
                ..Default::default()
            },
        ])),
    ]
}
