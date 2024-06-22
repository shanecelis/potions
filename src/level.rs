use super::*;

pub trait Level {
    fn potions(&self) -> &[Potion];
    fn is_complete(&self, potions: &[Potion]) -> bool;
}

pub struct UnmixLevel {
    potions: Vec<Potion>,
}

impl Level for UnmixLevel {

    fn potions(&self) -> &[Potion] {
        &self.potions
    }

    fn is_complete(&self, potions: &[Potion]) -> bool {
        potions.iter().all(|p| p.layers.len() <= 1)
    }
}

pub fn levels() -> Vec<Box<dyn Level>> {
    vec![
        Box::new(UnmixLevel {
            potions: vec![
                Potion {
                    layers: vec![
                        Layer::Liquid { color:
                                        color_art::Color::from_rgb(255, 0, 0).unwrap(),
                                        volume: 50.0 },

                        Layer::Liquid { color:
                                        color_art::Color::from_rgb(0, 255, 0).unwrap(),
                                        volume: 25.0 },
                    ],
                    ..Default::default()
                },

                Potion {
                    layers: vec![
                        Layer::Liquid { color:
                                        color_art::Color::from_rgb(0, 255, 0).unwrap(),
                                        volume: 50.0 },

                        Layer::Liquid { color:
                                        color_art::Color::from_rgb(0, 0, 255).unwrap(),
                                        volume: 25.0 },
                    ],
                    ..Default::default()
                },

                Potion {
                    layers: vec![
                        Layer::Liquid { color:
                                        color_art::Color::from_rgb(0, 0, 255).unwrap(),
                                        volume: 50.0 } ],
                    ..Default::default()
                },
            ]
        }),
    ]
}
