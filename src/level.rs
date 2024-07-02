use super::*;
use bevy_math::Vec2;
use derived_deref::{Deref, DerefMut};
use kolorwheel::{HslColor, KolorWheel, RgbColor, SpinMode};
use serde::{Deserialize, Serialize};
use std::collections::BinaryHeap;

#[derive(Debug, Clone, Deref, DerefMut, Deserialize, Serialize)]
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
            Layer::Liquid { id, .. } => self.0[*id].clone(),
        }
    }

    pub fn from_seed<T: Into<HslColor>>(color: T, count: usize) -> Self {
        let mut kw = KolorWheel::new(color, count);
        kw.with_hue(SpinMode::RelativeExcl(-360));
        let colors: Vec<color_art::Color> = kw
            .map(RgbColor::from)
            .map(|RgbColor { r, g, b }| color_art::Color::from_rgb(r, g, b).unwrap())
            .collect();
        Palette::new(colors)
    }
}

#[derive(Deserialize, Serialize)]
pub struct Level {
    pub palette: Palette,
    pub potions: Vec<Vial>,
    pub goal: Goal,
}

impl Default for Level {
    fn default() -> Self {
        Self {
            palette: Palette::new(vec![rgb(255, 0, 0), rgb(0, 255, 0), rgb(0, 0, 255)]),
            potions: vec![],
            goal: Goal::Unmix,
        }
    }
}

impl Level {
    /// Return unique layer IDs.
    fn layer_ids(vials: &[Vial]) -> impl Iterator<Item = usize> {
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
        heap.into_iter()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Goal {
    Unmix,
    BreakSeed,
}

impl Goal {
    pub fn is_complete(&self, potions: &[Vial]) -> bool {
        match self {
            Goal::Unmix => potions.iter().all(|p| p.layers.len() <= 1),
            Goal::BreakSeed => potions.iter().all(|p| {
                p.objects
                    .iter()
                    .filter(|o| matches!(o.kind, ObjectKind::Seed))
                    .map(|o| o.size)
                    .all(|s| s <= 1.0)
            }),
        }
    }
}

pub fn levels() -> Vec<Level> {
    vec![
        Level {
            goal: Goal::BreakSeed,
            potions: vec![
                Vial {
                    objects: vec![
                        // Object {
                        //     kind: ObjectKind::Seed,
                        //     pos: Vec2::new(4.9, 14.0),
                        //     size: 1.0,
                        //     id: 0,
                        // },
                        Object {
                            kind: ObjectKind::Seed,
                            pos: Vec2::new(10.0, 10.0),
                            size: 2.0,
                            id: 1,
                            ..Default::default()
                        },
                        // Object {
                        //     kind: ObjectKind::Seed,
                        //     pos: Vec2::new(15.0, 10.0),
                        //     size: 3.0,
                        //     id: 2,
                        // },
                        // Object {
                        //     kind: ObjectKind::Seed,
                        //     pos: Vec2::new(10.0, 2.0),
                        //     size: 4.0,
                        //     id: 3,
                        // },
                        // Object {
                        //     kind: ObjectKind::Seed,
                        //     pos: Vec2::new(20.0, 7.0),
                        //     size: 5.0,
                        //     id: 4,
                        // },
                    ],
                    ..Default::default()
                },
                Vial {
                    layers: vec![
                        Layer::Liquid {
                        id: 0,
                        volume: 50.0,
                    }
                    ],
                    ..Default::default()
                },
                Vial {
                    layers: vec![],
                    ..Default::default()
                },
            ],
            ..Default::default()
        },
        Level {
            potions: vec![
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
            ],
            ..Default::default()
        },
        Level {
            potions: vec![
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
            ],
            ..Default::default()
        },
        Level {
            potions: vec![
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
            ],
            ..Default::default()
        },
    ]
}
