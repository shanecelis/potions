use color_art::Color;
mod tui;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usize_modulus_substitute() {
        let count: usize = 5;
        let mut x: usize = count - 1;
        // We can increment x and use modulus to wrap with usize.
        assert_eq!((x + 1) % count, 0);
        assert_eq!((x - 1) % count, count - 2); // Can overflow.
        assert_eq!(x.checked_sub(1), Some(count - 2)); // Doesn't overflow.
        x = 0;
        // But we can't decrement x when it's 0.
        // assert_eq!((x - 1) % count, count - 1); // overflows!
        assert_eq!(x.checked_sub(1), None); // Check for None instead of overflow.
        // Not what we want:
        assert_eq!(x.saturating_sub(1) % count, 0);

        // Acceptable Substitutes
        // ======================
        assert_eq!(x.checked_sub(1).unwrap_or(count - 1), count - 1);
        assert_eq!((x + count - 1) % count, count - 1);
        assert_eq!((x + count - 1).rem_euclid(count), count - 1);
    }
}
