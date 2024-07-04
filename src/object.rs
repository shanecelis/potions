use approx::abs_diff_eq;
use derived_deref::{Deref};
use std::cmp::Ordering;
use bevy_math::Vec2;
use serde::{Deserialize, Serialize};

bitflags::bitflags! {
    #[derive(Debug, Clone, Deserialize, Serialize, Default)]
    pub struct ObjectFlags: u8 {
        const ENTER_VIAL = 0b00000001;
        const EXPECT_BREAK = 0b00000010;
        const BREAK = 0b00000100;
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Object {
    pub kind: ObjectKind,
    pub pos: Vec2,
    pub size: f32,
    pub id: u64,
    pub flags: ObjectFlags,
}

#[derive(Deref)]
pub struct ByHeight<'a>(pub usize, #[target] pub &'a Object);

impl<'a> PartialOrd for ByHeight<'a> {
    #[allow(clippy::non_canonical_partial_ord_impl)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.pos.y.partial_cmp(&other.pos.y)
    }
}

impl<'a> Ord for ByHeight<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.pos
            .y
            .partial_cmp(&other.pos.y)
            .or(abs_diff_eq!(self.pos.y, other.pos.y, epsilon = 0.1).then_some(Ordering::Equal))
            .unwrap_or(Ordering::Less)
    }
}

impl<'a> PartialEq for ByHeight<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.pos.y.eq(&other.pos.y)
    }
}

impl<'a> Eq for ByHeight<'a> {}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub enum ObjectKind {
    #[default]
    Seed,
    Creature,
    Plant,
}
