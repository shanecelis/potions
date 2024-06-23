use super::*;
use derived_deref::{Deref, DerefMut};
use std::collections::BinaryHeap;
use kolorwheel::{KolorWheel, RgbColor, SpinMode, HslColor};

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct Palette(Vec<Color>);

impl Palette {
    fn new<I, J>(iter: I) -> Self where I: IntoIterator<Item = J>,
    J: Into<Color> {
        Self(iter.into_iter().map(|x| x.into()).collect())
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
}

impl Default for UnmixLevel {

    fn default() -> Self {
        Self {
            palette: Palette::new(vec![
                color_art::Color::from_rgb(255, 0, 0).unwrap(),
                color_art::Color::from_rgb(0, 255, 0).unwrap(),
                color_art::Color::from_rgb(0, 0, 255).unwrap(),
            ]),
            potions: vec![],
        }
    }
}

impl UnmixLevel {
    fn new(vials: Vec<Vial>) -> Self {
        let heap: BinaryHeap<usize> = vials.iter().flat_map(|v| &v.layers).filter_map(|l| if let Layer::Liquid {id, ..} = l { Some(*id) } else { None }).collect();
        // let mut kw = KolorWheel::new(RgbColor { r: 255, g: 0, b: 0 }, heap.len());
        // let mut kw = KolorWheel::new(HslColor { h: 185.0, s: 70.0, l: 65.0 }, heap.len());
        let mut kw = KolorWheel::new(HslColor { h: 240.0, s: 82.0, l: 64.0 }, heap.len());
        kw.with_hue(SpinMode::RelativeExcl(-360));
        let colors: Vec<color_art::Color> = kw.map(|c| RgbColor::from(c)).map(|RgbColor { r,g,b }| color_art::Color::from_rgb(r, g, b).unwrap()).collect();
        assert_eq!(colors.len(), heap.len());
        Self {
            palette: Palette::new(colors),
            potions: vials,
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
        potions.iter().all(|p| p.layers.len() <= 1)
    }
}

pub fn levels() -> Vec<Box<dyn Level>> {
    vec![

        Box::new(UnmixLevel::new(
            vec![
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
                    ],
                    ..Default::default()
                },
            ],
        )),
        Box::new(UnmixLevel::new(
            vec![
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
        )),

        Box::new(UnmixLevel::new(
            vec![
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
            ]
        )),
    ]
}
