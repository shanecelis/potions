pub mod constant;
mod level;
pub mod tui;
mod vial;
pub mod vial_physics;

pub use level::{levels, Level, Palette};
pub use vial::{Color, Layer, Lerp, Object, ObjectKind, Transfer, Transition, Vial, VialLoc};

#[cfg(test)]
mod tests {

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
