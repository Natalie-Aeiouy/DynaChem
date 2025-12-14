// Integration tests for Dynachem
// Tests the interaction between physics, particles, and input systems

use dynachem::physics::constants::*;
use dynachem::physics::coulomb::coulomb_force;
use dynachem::physics::simulation::{verlet_position_step, verlet_velocity_step, kinetic_energy};
use dynachem::particles::proton::Proton;
use dynachem::particles::electron::Electron;
use dynachem::input::spring::{spring_force, SpringConfig};
use glam::DVec3;
use approx::assert_relative_eq;

/// Test a complete hydrogen atom simulation:
/// An electron orbiting a proton under Coulomb attraction.
#[test]
fn hydrogen_atom_orbit_stability() {
    // Proton at origin (stationary - much heavier than electron)
    let proton_pos = DVec3::ZERO;

    // Electron at 1 Bohr radius, with circular orbit velocity
    // For circular orbit: v = sqrt(ke²/(m*r))
    let r = BOHR_RADIUS;
    let orbital_v = (COULOMB_CONSTANT * ELEMENTARY_CHARGE.powi(2)
        / (ELECTRON_MASS * r)).sqrt();

    let mut electron = Electron::new(DVec3::new(r, 0.0, 0.0));
    electron.velocity = DVec3::new(0.0, orbital_v, 0.0);

    // Calculate initial force
    let force = coulomb_force(
        Electron::charge(),
        Proton::charge(),
        electron.position,
        proton_pos
    );
    electron.force = force;

    // Initial energy
    let initial_ke = kinetic_energy(&electron);
    let initial_pe = COULOMB_CONSTANT * Electron::charge() * Proton::charge() / r;
    let initial_total = initial_ke + initial_pe;

    // Simulate for 1/4 orbit
    let dt = 1.0e-19;
    let steps = 1000;

    for _ in 0..steps {
        let old_accel = verlet_position_step(&mut electron, dt);

        // Recalculate Coulomb force
        electron.force = coulomb_force(
            Electron::charge(),
            Proton::charge(),
            electron.position,
            proton_pos
        );

        verlet_velocity_step(&mut electron, old_accel, dt);
    }

    // Check energy conservation
    let final_ke = kinetic_energy(&electron);
    let final_r = electron.position.length();
    let final_pe = COULOMB_CONSTANT * Electron::charge() * Proton::charge() / final_r;
    let final_total = final_ke + final_pe;

    // Energy should be conserved in orbit
    assert_relative_eq!(final_total, initial_total, max_relative = 0.01);

    // Orbit should remain approximately circular
    assert_relative_eq!(final_r, r, max_relative = 0.1);
}

/// Test spring-dragged proton interacting with electron cloud.
/// The proton is dragged by a virtual spring while experiencing
/// Coulomb attraction from the electron.
#[test]
fn spring_drag_with_coulomb_force() {
    // Set up spring config with reasonable values
    let spring_config = SpringConfig {
        stiffness: 1.0e-7,  // Appropriate for atomic-scale forces
        damping: 1.0e-14,
        max_force: 1.0e-6,
    };

    // Electron fixed at origin (simulating a much heavier nucleus or fixed point)
    let electron_pos = DVec3::ZERO;

    // Proton starts 2 Ångströms away
    let mut proton = Proton::new(DVec3::new(2.0 * ANGSTROM, 0.0, 0.0));

    // User drags finger to pull proton away (to 3 Ångströms)
    let target_pos = DVec3::new(3.0 * ANGSTROM, 0.0, 0.0);

    // Calculate both forces
    let coulomb_f = coulomb_force(
        Proton::charge(),
        Electron::charge(),
        proton.position,
        electron_pos
    );

    let spring_f = spring_force(
        proton.position,
        proton.velocity,
        target_pos,
        &spring_config
    );

    // Coulomb force should pull toward electron (negative x)
    assert!(coulomb_f.x < 0.0, "Coulomb should attract proton toward electron");

    // Spring force should pull toward target (positive x, away from electron)
    assert!(spring_f.x > 0.0, "Spring should pull proton toward drag target");

    // The proton experiences both forces
    proton.force = coulomb_f + spring_f;

    // With strong enough spring, net force should be toward target
    // With this setup, spring is pulling away while Coulomb pulls back
    // The equilibrium position depends on force balance
}

/// Test that two protons repel each other correctly.
#[test]
fn two_proton_repulsion() {
    let mut proton1 = Proton::new(DVec3::new(-ANGSTROM, 0.0, 0.0));
    let mut proton2 = Proton::new(DVec3::new(ANGSTROM, 0.0, 0.0));

    // Calculate mutual forces
    let force_on_1 = coulomb_force(
        Proton::charge(),
        Proton::charge(),
        proton1.position,
        proton2.position
    );
    let force_on_2 = coulomb_force(
        Proton::charge(),
        Proton::charge(),
        proton2.position,
        proton1.position
    );

    proton1.force = force_on_1;
    proton2.force = force_on_2;

    // Forces should be equal and opposite (Newton's 3rd law)
    assert_relative_eq!(force_on_1.length(), force_on_2.length(), max_relative = 0.001);
    assert_relative_eq!(force_on_1.x, -force_on_2.x, max_relative = 0.001);

    // Simulate a few steps
    let dt = 1.0e-18;
    for _ in 0..100 {
        let old_accel_1 = verlet_position_step(&mut proton1, dt);
        let old_accel_2 = verlet_position_step(&mut proton2, dt);

        // Recalculate forces
        proton1.force = coulomb_force(
            Proton::charge(), Proton::charge(),
            proton1.position, proton2.position
        );
        proton2.force = coulomb_force(
            Proton::charge(), Proton::charge(),
            proton2.position, proton1.position
        );

        verlet_velocity_step(&mut proton1, old_accel_1, dt);
        verlet_velocity_step(&mut proton2, old_accel_2, dt);
    }

    // Protons should have moved apart (repulsion)
    let final_distance = (proton2.position - proton1.position).length();
    let initial_distance = 2.0 * ANGSTROM;
    assert!(final_distance > initial_distance,
        "Protons should repel and move apart");
}

/// Test the probability cloud correctly represents electron distribution.
#[test]
fn electron_probability_cloud_physics() {
    use dynachem::particles::electron::ProbabilityCloud;

    let cloud = ProbabilityCloud::hydrogen_1s(DVec3::ZERO);

    // Most probable location is at the nucleus (for 1s orbital density)
    let prob_at_center = cloud.probability_density(DVec3::ZERO);
    let prob_at_bohr = cloud.probability_density(DVec3::new(BOHR_RADIUS, 0.0, 0.0));

    assert!(prob_at_center > prob_at_bohr,
        "1s orbital has maximum probability density at nucleus");

    // But the RADIAL probability (probability of finding electron at distance r)
    // is maximum at the Bohr radius due to the r² factor in spherical coordinates
    // P(r) = 4πr² |ψ(r)|²
    let radial_prob_at_bohr = 4.0 * std::f64::consts::PI
        * BOHR_RADIUS.powi(2) * prob_at_bohr;
    let radial_prob_at_half_bohr = 4.0 * std::f64::consts::PI
        * (0.5 * BOHR_RADIUS).powi(2)
        * cloud.probability_density(DVec3::new(0.5 * BOHR_RADIUS, 0.0, 0.0));

    assert!(radial_prob_at_bohr > radial_prob_at_half_bohr,
        "Radial probability should peak near Bohr radius");
}
