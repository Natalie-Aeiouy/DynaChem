// Proton component
// A proton has charge +e and mass ~1836 times the electron mass

use bevy::prelude::*;
use glam::DVec3;
use crate::physics::constants::{ELEMENTARY_CHARGE, PROTON_MASS};

/// A proton particle component.
/// Protons have positive charge and are found in atomic nuclei.
#[derive(Component, Debug, Clone)]
pub struct Proton {
    /// Position in meters (SI units)
    pub position: DVec3,
    /// Velocity in meters per second
    pub velocity: DVec3,
    /// Accumulated force in Newtons (reset each physics step)
    pub force: DVec3,
}

impl Proton {
    /// Create a new proton at the given position, initially at rest.
    pub fn new(position: DVec3) -> Self {
        Self {
            position,
            velocity: DVec3::ZERO,
            force: DVec3::ZERO,
        }
    }

    /// Create a new proton with initial velocity.
    pub fn with_velocity(position: DVec3, velocity: DVec3) -> Self {
        Self {
            position,
            velocity,
            force: DVec3::ZERO,
        }
    }

    /// Returns the charge of a proton in Coulombs (+e)
    #[inline]
    pub fn charge() -> f64 {
        ELEMENTARY_CHARGE
    }

    /// Returns the mass of a proton in kilograms
    #[inline]
    pub fn mass() -> f64 {
        PROTON_MASS
    }

    /// Add a force to the accumulated force on this proton.
    pub fn apply_force(&mut self, force: DVec3) {
        self.force += force;
    }

    /// Clear accumulated forces (call after integration step)
    pub fn clear_forces(&mut self) {
        self.force = DVec3::ZERO;
    }
}

impl Default for Proton {
    fn default() -> Self {
        Self::new(DVec3::ZERO)
    }
}

// Implement Integratable trait for use with Velocity Verlet simulation
impl crate::physics::simulation::Integratable for Proton {
    fn position(&self) -> DVec3 { self.position }
    fn velocity(&self) -> DVec3 { self.velocity }
    fn force(&self) -> DVec3 { self.force }
    fn mass(&self) -> f64 { Self::mass() }

    fn set_position(&mut self, pos: DVec3) { self.position = pos; }
    fn set_velocity(&mut self, vel: DVec3) { self.velocity = vel; }
    fn clear_forces(&mut self) { self.force = DVec3::ZERO; }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::constants::{ELEMENTARY_CHARGE, PROTON_MASS, ELECTRON_MASS};
    use approx::assert_relative_eq;

    #[test]
    fn proton_has_positive_charge() {
        assert!(Proton::charge() > 0.0, "Proton should have positive charge");
        assert_relative_eq!(Proton::charge(), ELEMENTARY_CHARGE, epsilon = 1e-30);
    }

    #[test]
    fn proton_mass_is_correct() {
        assert_relative_eq!(Proton::mass(), PROTON_MASS, epsilon = 1e-40);
    }

    #[test]
    fn proton_mass_ratio_to_electron() {
        // Proton is approximately 1836 times heavier than electron
        let ratio = Proton::mass() / ELECTRON_MASS;
        assert_relative_eq!(ratio, 1836.15, epsilon = 0.01);
    }

    #[test]
    fn new_proton_at_rest() {
        let pos = DVec3::new(1.0, 2.0, 3.0);
        let proton = Proton::new(pos);

        assert_eq!(proton.position, pos);
        assert_eq!(proton.velocity, DVec3::ZERO);
        assert_eq!(proton.force, DVec3::ZERO);
    }

    #[test]
    fn proton_with_velocity() {
        let pos = DVec3::new(1.0, 0.0, 0.0);
        let vel = DVec3::new(0.0, 100.0, 0.0);
        let proton = Proton::with_velocity(pos, vel);

        assert_eq!(proton.position, pos);
        assert_eq!(proton.velocity, vel);
    }

    #[test]
    fn apply_forces_accumulate() {
        let mut proton = Proton::default();

        proton.apply_force(DVec3::new(1.0, 0.0, 0.0));
        proton.apply_force(DVec3::new(0.0, 2.0, 0.0));

        assert_eq!(proton.force, DVec3::new(1.0, 2.0, 0.0));
    }

    #[test]
    fn clear_forces_resets_to_zero() {
        let mut proton = Proton::default();
        proton.apply_force(DVec3::new(1.0, 2.0, 3.0));
        proton.clear_forces();

        assert_eq!(proton.force, DVec3::ZERO);
    }
}
