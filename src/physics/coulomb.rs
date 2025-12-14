// Coulomb force calculation
// F = k * q1 * q2 / r²

use glam::DVec3;
use super::constants::COULOMB_CONSTANT;

/// Calculate the Coulomb force between two point charges.
///
/// # Arguments
/// * `q1` - First charge in Coulombs
/// * `q2` - Second charge in Coulombs
/// * `r1` - Position of first charge in meters
/// * `r2` - Position of second charge in meters
///
/// # Returns
/// Force vector on charge 1 due to charge 2, in Newtons.
/// Positive (repulsive) when charges have same sign.
/// Negative (attractive) when charges have opposite signs.
pub fn coulomb_force(q1: f64, q2: f64, r1: DVec3, r2: DVec3) -> DVec3 {
    let displacement = r1 - r2;
    let distance = displacement.length();

    assert!(distance > 0.0, "Cannot calculate Coulomb force at zero distance (singularity)");

    // F = k * q1 * q2 / r² * r̂
    // where r̂ is unit vector from r2 to r1
    let magnitude = COULOMB_CONSTANT * q1 * q2 / (distance * distance);
    let direction = displacement / distance; // unit vector

    direction * magnitude
}

/// Calculate the magnitude of Coulomb force between two point charges.
///
/// # Arguments
/// * `q1` - First charge in Coulombs
/// * `q2` - Second charge in Coulombs
/// * `distance` - Distance between charges in meters
///
/// # Returns
/// Magnitude of force in Newtons. Positive for repulsion, negative for attraction.
pub fn coulomb_force_magnitude(q1: f64, q2: f64, distance: f64) -> f64 {
    assert!(distance > 0.0, "Cannot calculate Coulomb force at zero distance (singularity)");

    // F = k * q1 * q2 / r²
    // Positive result = repulsion (same sign charges)
    // Negative result = attraction (opposite sign charges)
    COULOMB_CONSTANT * q1 * q2 / (distance * distance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::constants::{ELEMENTARY_CHARGE, ANGSTROM};
    use approx::assert_relative_eq;

    #[test]
    fn proton_electron_force_at_one_angstrom() {
        // A proton and electron separated by 1 Ångström
        // F = k * e * (-e) / r²
        // F = 8.99e9 * (1.6e-19)² / (1e-10)²
        // F ≈ 2.3e-8 N (attractive, so negative for magnitude convention)

        let proton_charge = ELEMENTARY_CHARGE;
        let electron_charge = -ELEMENTARY_CHARGE;
        let distance = ANGSTROM;

        let force_magnitude = coulomb_force_magnitude(proton_charge, electron_charge, distance);

        // Expected: -2.307e-8 N (negative = attractive)
        let expected = -2.307e-8;
        assert_relative_eq!(force_magnitude, expected, epsilon = 1e-10);
    }

    #[test]
    fn two_protons_repel() {
        // Two protons at 1 Ångström should repel
        let q1 = ELEMENTARY_CHARGE;
        let q2 = ELEMENTARY_CHARGE;
        let distance = ANGSTROM;

        let force = coulomb_force_magnitude(q1, q2, distance);

        // Positive = repulsive
        assert!(force > 0.0, "Same charges should repel (positive force)");
        assert_relative_eq!(force, 2.307e-8, epsilon = 1e-10);
    }

    #[test]
    fn force_vector_points_correctly_for_attraction() {
        // Electron at origin, proton at (1Å, 0, 0)
        // Force on proton should point toward electron (negative x direction)
        let electron_pos = DVec3::ZERO;
        let proton_pos = DVec3::new(ANGSTROM, 0.0, 0.0);

        let force_on_proton = coulomb_force(
            ELEMENTARY_CHARGE,      // proton charge
            -ELEMENTARY_CHARGE,     // electron charge
            proton_pos,
            electron_pos,
        );

        // Force should point in negative x direction (toward electron)
        assert!(force_on_proton.x < 0.0, "Proton should be attracted toward electron");
        assert_relative_eq!(force_on_proton.y, 0.0, epsilon = 1e-30);
        assert_relative_eq!(force_on_proton.z, 0.0, epsilon = 1e-30);
    }

    #[test]
    fn force_vector_points_correctly_for_repulsion() {
        // Two protons: one at origin, one at (1Å, 0, 0)
        // Force on second proton should point away from first (positive x direction)
        let p1_pos = DVec3::ZERO;
        let p2_pos = DVec3::new(ANGSTROM, 0.0, 0.0);

        let force_on_p2 = coulomb_force(
            ELEMENTARY_CHARGE,  // p2 charge
            ELEMENTARY_CHARGE,  // p1 charge
            p2_pos,
            p1_pos,
        );

        // Force should point in positive x direction (away from p1)
        assert!(force_on_p2.x > 0.0, "Proton should be repelled from other proton");
    }

    #[test]
    fn inverse_square_law() {
        // Force at 2Å should be 1/4 the force at 1Å
        let q1 = ELEMENTARY_CHARGE;
        let q2 = -ELEMENTARY_CHARGE;

        let force_1a = coulomb_force_magnitude(q1, q2, ANGSTROM).abs();
        let force_2a = coulomb_force_magnitude(q1, q2, 2.0 * ANGSTROM).abs();

        let ratio = force_1a / force_2a;
        assert_relative_eq!(ratio, 4.0, epsilon = 1e-10);
    }

    #[test]
    #[should_panic]
    fn zero_distance_panics() {
        // Singularity at r=0 should panic
        coulomb_force_magnitude(ELEMENTARY_CHARGE, ELEMENTARY_CHARGE, 0.0);
    }
}
