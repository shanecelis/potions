use color_art::Color;
mod tui;

#[derive(Debug, Clone)]
pub struct Potion {
    pub layers: Vec<Layer>,
    pub volume: f64,
    pub glass: Color,
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
}
