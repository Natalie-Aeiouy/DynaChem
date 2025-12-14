// Electron component with probability cloud representation
// An electron has charge -e and exists as a probability cloud around nuclei

use bevy::prelude::*;
use glam::DVec3;
use crate::physics::constants::{ELEMENTARY_CHARGE, ELECTRON_MASS, BOHR_RADIUS};

/// An electron particle component.
/// Unlike classical particles, electrons exist as probability clouds.
#[derive(Component, Debug, Clone)]
pub struct Electron {
    /// Mean position of the probability cloud center in meters
    pub position: DVec3,
    /// Velocity of the cloud center in meters per second
    pub velocity: DVec3,
    /// Accumulated force in Newtons
    pub force: DVec3,
}

impl Electron {
    /// Create a new electron at the given position.
    pub fn new(position: DVec3) -> Self {
        Self {
            position,
            velocity: DVec3::ZERO,
            force: DVec3::ZERO,
        }
    }

    /// Create a new electron with initial velocity.
    pub fn with_velocity(position: DVec3, velocity: DVec3) -> Self {
        Self {
            position,
            velocity,
            force: DVec3::ZERO,
        }
    }

    /// Returns the charge of an electron in Coulombs (-e)
    #[inline]
    pub fn charge() -> f64 {
        -ELEMENTARY_CHARGE
    }

    /// Returns the mass of an electron in kilograms
    #[inline]
    pub fn mass() -> f64 {
        ELECTRON_MASS
    }

    /// Add a force to the accumulated force on this electron.
    pub fn apply_force(&mut self, force: DVec3) {
        self.force += force;
    }

    /// Clear accumulated forces
    pub fn clear_forces(&mut self) {
        self.force = DVec3::ZERO;
    }
}

impl Default for Electron {
    fn default() -> Self {
        Self::new(DVec3::ZERO)
    }
}

// Implement Integratable trait for use with Velocity Verlet simulation
impl crate::physics::simulation::Integratable for Electron {
    fn position(&self) -> DVec3 { self.position }
    fn velocity(&self) -> DVec3 { self.velocity }
    fn force(&self) -> DVec3 { self.force }
    fn mass(&self) -> f64 { Self::mass() }

    fn set_position(&mut self, pos: DVec3) { self.position = pos; }
    fn set_velocity(&mut self, vel: DVec3) { self.velocity = vel; }
    fn clear_forces(&mut self) { self.force = DVec3::ZERO; }
}

/// Represents the probability cloud (wavefunction) of an electron.
/// This determines the spatial distribution of electron probability.
#[derive(Component, Debug, Clone)]
pub struct ProbabilityCloud {
    /// The type of orbital (determines shape)
    pub orbital: OrbitalType,
    /// Characteristic length scale (e.g., Bohr radius for 1s)
    pub length_scale: f64,
    /// Center position of the cloud (typically centered on a nucleus)
    pub center: DVec3,
}

/// Types of atomic orbitals with different shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrbitalType {
    /// Spherical orbital (1s, 2s, etc.)
    S { n: u32 },
    /// Dumbbell-shaped orbital (2p, 3p, etc.)
    P { n: u32, m: i32 },
    /// More complex shapes (3d, 4d, etc.)
    D { n: u32, m: i32 },
}

impl ProbabilityCloud {
    /// Create a ground state hydrogen 1s orbital.
    pub fn hydrogen_1s(center: DVec3) -> Self {
        Self {
            orbital: OrbitalType::S { n: 1 },
            length_scale: BOHR_RADIUS,
            center,
        }
    }

    /// Calculate the probability density at a given point.
    /// Returns |ψ|² (probability per unit volume).
    pub fn probability_density(&self, point: DVec3) -> f64 {
        let r = (point - self.center).length();

        match self.orbital {
            OrbitalType::S { n: 1 } => {
                // 1s orbital: |ψ|² = (1/πa₀³) * e^(-2r/a₀)
                let a0 = self.length_scale;
                let normalization = 1.0 / (std::f64::consts::PI * a0.powi(3));
                normalization * (-2.0 * r / a0).exp()
            }
            OrbitalType::S { n: 2 } => {
                // 2s orbital: more complex radial function
                let a0 = self.length_scale;
                let rho = r / a0;
                let radial = (2.0 - rho) * (-rho / 2.0).exp();
                let normalization = 1.0 / (32.0 * std::f64::consts::PI * a0.powi(3));
                normalization * radial.powi(2)
            }
            // Higher orbitals can be added as needed
            _ => {
                // Fallback to 1s-like behavior for unimplemented orbitals
                let a0 = self.length_scale;
                let normalization = 1.0 / (std::f64::consts::PI * a0.powi(3));
                normalization * (-2.0 * r / a0).exp()
            }
        }
    }

    /// Get the radius at which probability density is some fraction of maximum.
    /// Useful for determining visual cloud extent.
    pub fn extent_radius(&self, fraction: f64) -> f64 {
        match self.orbital {
            OrbitalType::S { n: 1 } => {
                // For 1s: |ψ|² ∝ e^(-2r/a₀)
                // Solve e^(-2r/a₀) = fraction
                // r = -a₀ * ln(fraction) / 2
                -self.length_scale * fraction.ln() / 2.0
            }
            _ => {
                // Approximate for other orbitals
                -self.length_scale * fraction.ln() / 2.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::constants::{ELEMENTARY_CHARGE, ELECTRON_MASS, BOHR_RADIUS};
    use approx::assert_relative_eq;

    #[test]
    fn electron_has_negative_charge() {
        assert!(Electron::charge() < 0.0, "Electron should have negative charge");
        assert_relative_eq!(Electron::charge(), -ELEMENTARY_CHARGE, epsilon = 1e-30);
    }

    #[test]
    fn electron_mass_is_correct() {
        assert_relative_eq!(Electron::mass(), ELECTRON_MASS, epsilon = 1e-40);
    }

    #[test]
    fn opposite_charges() {
        // Electron and proton should have equal and opposite charges
        use crate::particles::proton::Proton;
        assert_relative_eq!(
            Electron::charge() + Proton::charge(),
            0.0,
            epsilon = 1e-30
        );
    }

    #[test]
    fn hydrogen_1s_probability_at_origin() {
        let cloud = ProbabilityCloud::hydrogen_1s(DVec3::ZERO);

        // At r=0, |ψ|² = 1/(π * a₀³)
        let expected = 1.0 / (std::f64::consts::PI * BOHR_RADIUS.powi(3));
        let actual = cloud.probability_density(DVec3::ZERO);

        assert_relative_eq!(actual, expected, epsilon = 1e-10);
    }

    #[test]
    fn hydrogen_1s_probability_decreases_with_distance() {
        let cloud = ProbabilityCloud::hydrogen_1s(DVec3::ZERO);

        let prob_at_origin = cloud.probability_density(DVec3::ZERO);
        let prob_at_bohr = cloud.probability_density(DVec3::new(BOHR_RADIUS, 0.0, 0.0));
        let prob_at_2bohr = cloud.probability_density(DVec3::new(2.0 * BOHR_RADIUS, 0.0, 0.0));

        assert!(prob_at_bohr < prob_at_origin, "Probability should decrease with distance");
        assert!(prob_at_2bohr < prob_at_bohr, "Probability should continue decreasing");
    }

    #[test]
    fn hydrogen_1s_probability_at_bohr_radius() {
        let cloud = ProbabilityCloud::hydrogen_1s(DVec3::ZERO);

        // At r = a₀: |ψ|² = (1/πa₀³) * e^(-2)
        let expected = 1.0 / (std::f64::consts::PI * BOHR_RADIUS.powi(3)) * (-2.0_f64).exp();
        let actual = cloud.probability_density(DVec3::new(BOHR_RADIUS, 0.0, 0.0));

        assert_relative_eq!(actual, expected, epsilon = 1e-10);
    }

    #[test]
    fn probability_is_spherically_symmetric_for_s_orbital() {
        let cloud = ProbabilityCloud::hydrogen_1s(DVec3::ZERO);
        let r = BOHR_RADIUS;

        let prob_x = cloud.probability_density(DVec3::new(r, 0.0, 0.0));
        let prob_y = cloud.probability_density(DVec3::new(0.0, r, 0.0));
        let prob_z = cloud.probability_density(DVec3::new(0.0, 0.0, r));
        let prob_diag = cloud.probability_density(DVec3::new(r / 3.0_f64.sqrt(), r / 3.0_f64.sqrt(), r / 3.0_f64.sqrt()));

        // Use relative epsilon for large numbers
        assert_relative_eq!(prob_x, prob_y, max_relative = 1e-10);
        assert_relative_eq!(prob_y, prob_z, max_relative = 1e-10);
        assert_relative_eq!(prob_z, prob_diag, max_relative = 1e-10);
    }

    #[test]
    fn extent_radius_increases_for_smaller_fractions() {
        let cloud = ProbabilityCloud::hydrogen_1s(DVec3::ZERO);

        let r_10_percent = cloud.extent_radius(0.1);
        let r_1_percent = cloud.extent_radius(0.01);

        assert!(r_1_percent > r_10_percent, "Smaller fraction should give larger radius");
    }

    #[test]
    fn probability_cloud_normalization() {
        // The integral of |ψ|² over all space should equal 1.
        // We approximate this with numerical integration.
        let cloud = ProbabilityCloud::hydrogen_1s(DVec3::ZERO);

        // Integrate in spherical coordinates: ∫|ψ|² 4πr² dr from 0 to ∞
        // For 1s: ∫(1/πa₀³)e^(-2r/a₀) * 4πr² dr
        // = (4/a₀³) ∫r² e^(-2r/a₀) dr from 0 to ∞
        // = (4/a₀³) * (a₀/2)³ * 2! = (4/a₀³) * (a₀³/8) * 2 = 1

        // Numerical check with trapezoid rule
        let n_points = 1000;
        let r_max = 10.0 * BOHR_RADIUS;
        let dr = r_max / n_points as f64;

        let mut integral = 0.0;
        for i in 0..n_points {
            let r = (i as f64 + 0.5) * dr;
            let prob = cloud.probability_density(DVec3::new(r, 0.0, 0.0));
            integral += prob * 4.0 * std::f64::consts::PI * r * r * dr;
        }

        // Should be close to 1 (some numerical error expected)
        assert_relative_eq!(integral, 1.0, epsilon = 0.01);
    }
}
