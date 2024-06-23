use color_art::Color;

#[derive(Debug, Clone)]
pub struct Potion {
    pub layers: Vec<Layer>,
    pub volume: f64,
    pub glass: Color,
}

impl Potion {
    pub fn top_layer(&self) -> Option<&Layer> {
        self.layers.get(self.layers.len() - 1)
    }

    pub fn vol(&self) -> f64 {
        self.layers.iter().map(|l: &Layer| l.volume()).sum()
    }

    /// Pour self into other potion.
    pub fn pour(&self, other: &Potion) -> Option<(Potion, Potion)> {
        self.top_layer()
            .and_then(|a| other.top_layer().and_then(|b| {
                match (a, b) {
                    (&Layer::Liquid { color: color_a, volume: volume_a },
                     &Layer::Liquid { color: color_b, volume: volume_b }) =>
                        if color_a == color_b {
                            let empty_volume = other.volume - other.vol();
                            if empty_volume > 0.0 {
                                let mut s = self.clone();
                                let mut o = other.clone();
                                if volume_a > empty_volume {
                                    // We pour some.
                                    *s.layers.last_mut().unwrap() = Layer::Liquid {
                                        volume: volume_a - empty_volume,
                                        color: color_a
                                    };

                                    *o.layers.last_mut().unwrap() = Layer::Liquid {
                                        volume: volume_b + empty_volume,
                                        color: color_b
                                    };
                                } else {
                                    // We pour all.
                                    s.layers.pop();

                                    *o.layers.last_mut().unwrap() = Layer::Liquid {
                                        volume: volume_b + volume_a,
                                        color: color_b
                                    };
                                }
                                Some((s, o))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    _ => None
                }

            }))
    }
}

impl Default for Potion {
    fn default() -> Self {
        Self {
            layers: vec![],
            volume: 100.0,
            glass: Color::from_rgba(255, 255, 255, 0.5).unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Layer {
    Liquid { color: Color, volume: f64 },
    Object(Object),
    Empty,
}

impl Layer {
    pub fn volume(&self) -> f64 {
        match self {
            Layer::Liquid { volume, .. } => *volume,
            Layer::Object(_) => todo!(),
            Layer::Empty => 0.0
        }
    }
}

#[derive(Debug, Clone)]
pub enum Object {
    Seed,
    BrokenSeeds,
    Creature,
    Plant,
}