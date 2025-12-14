// Time stepping and force integration using Velocity Verlet
// Velocity Verlet is symplectic and stable for oscillatory systems

use glam::DVec3;

/// Configuration for the physics simulation.
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    /// Time step in seconds
    pub dt: f64,
    /// Time scale multiplier (1.0 = real time, 1e12 = 1 femtosecond per millisecond)
    pub time_scale: f64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            // Default to 1 femtosecond timestep (appropriate for atomic motion)
            dt: 1.0e-15,
            // Default time scale: 1e12 means simulation runs at ~1 femtosecond per millisecond
            // At 60fps, each frame advances ~16.7 femtoseconds of simulation time
            time_scale: 1.0e12,
        }
    }
}

impl SimulationConfig {
    /// Create a new configuration with specified timestep
    pub fn with_dt(dt: f64) -> Self {
        Self { dt, ..Default::default() }
    }

    /// Effective timestep accounting for time scale
    pub fn effective_dt(&self) -> f64 {
        self.dt * self.time_scale
    }
}

/// A generic particle that can be integrated with Velocity Verlet.
/// This trait allows the same integration code to work with any particle type.
pub trait Integratable {
    fn position(&self) -> DVec3;
    fn velocity(&self) -> DVec3;
    fn force(&self) -> DVec3;
    fn mass(&self) -> f64;

    fn set_position(&mut self, pos: DVec3);
    fn set_velocity(&mut self, vel: DVec3);
    fn clear_forces(&mut self);
}

/// Velocity Verlet integration step.
///
/// The algorithm:
/// 1. x(t+dt) = x(t) + v(t)*dt + 0.5*a(t)*dt²
/// 2. (caller recalculates forces at new position)
/// 3. v(t+dt) = v(t) + 0.5*(a(t) + a(t+dt))*dt
///
/// This function performs step 1 (position update) and stores the old acceleration.
/// After forces are recalculated, call `verlet_velocity_step` to complete.
pub fn verlet_position_step<T: Integratable>(particle: &mut T, dt: f64) -> DVec3 {
    let pos = particle.position();
    let vel = particle.velocity();
    let force = particle.force();
    let mass = particle.mass();

    // Current acceleration
    let accel = force / mass;

    // Update position: x(t+dt) = x(t) + v(t)*dt + 0.5*a(t)*dt²
    let new_pos = pos + vel * dt + 0.5 * accel * dt * dt;
    particle.set_position(new_pos);

    // Return old acceleration for velocity step
    accel
}

/// Complete the Velocity Verlet step by updating velocity.
/// Call this after forces have been recalculated at the new position.
///
/// # Arguments
/// * `particle` - The particle to update
/// * `old_accel` - Acceleration from before the position step
/// * `dt` - Time step in seconds
pub fn verlet_velocity_step<T: Integratable>(particle: &mut T, old_accel: DVec3, dt: f64) {
    let vel = particle.velocity();
    let force = particle.force();
    let mass = particle.mass();

    // New acceleration at updated position
    let new_accel = force / mass;

    // Update velocity: v(t+dt) = v(t) + 0.5*(a(t) + a(t+dt))*dt
    let new_vel = vel + 0.5 * (old_accel + new_accel) * dt;
    particle.set_velocity(new_vel);
}

/// Perform a complete Velocity Verlet step for a single particle.
/// This is a convenience function that combines both steps.
///
/// Note: This only works for systems where force doesn't depend on velocity.
/// For velocity-dependent forces, use the two-step process.
pub fn verlet_full_step<T: Integratable, F>(particle: &mut T, dt: f64, force_calc: F)
where
    F: FnOnce(DVec3) -> DVec3, // Takes position, returns force
{
    // Step 1: Update position
    let old_accel = verlet_position_step(particle, dt);

    // Recalculate force at new position
    particle.clear_forces();
    let new_force = force_calc(particle.position());
    // We need to manually apply the force since we cleared it
    let mass = particle.mass();
    let new_accel = new_force / mass;

    // Step 2: Update velocity
    let vel = particle.velocity();
    let new_vel = vel + 0.5 * (old_accel + new_accel) * dt;
    particle.set_velocity(new_vel);
}

/// Calculate kinetic energy of a particle
pub fn kinetic_energy<T: Integratable>(particle: &T) -> f64 {
    let vel = particle.velocity();
    0.5 * particle.mass() * vel.length_squared()
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    /// Simple test particle for unit tests
    #[derive(Debug, Clone)]
    struct TestParticle {
        position: DVec3,
        velocity: DVec3,
        force: DVec3,
        mass: f64,
    }

    impl TestParticle {
        fn new(mass: f64) -> Self {
            Self {
                position: DVec3::ZERO,
                velocity: DVec3::ZERO,
                force: DVec3::ZERO,
                mass,
            }
        }

        fn at(mut self, pos: DVec3) -> Self {
            self.position = pos;
            self
        }

        fn with_velocity(mut self, vel: DVec3) -> Self {
            self.velocity = vel;
            self
        }

        fn with_force(mut self, force: DVec3) -> Self {
            self.force = force;
            self
        }
    }

    impl Integratable for TestParticle {
        fn position(&self) -> DVec3 { self.position }
        fn velocity(&self) -> DVec3 { self.velocity }
        fn force(&self) -> DVec3 { self.force }
        fn mass(&self) -> f64 { self.mass }

        fn set_position(&mut self, pos: DVec3) { self.position = pos; }
        fn set_velocity(&mut self, vel: DVec3) { self.velocity = vel; }
        fn clear_forces(&mut self) { self.force = DVec3::ZERO; }
    }

    #[test]
    fn config_default_has_femtosecond_timestep() {
        let config = SimulationConfig::default();
        assert_relative_eq!(config.dt, 1.0e-15, max_relative = 0.01);
    }

    #[test]
    fn free_particle_moves_in_straight_line() {
        // A particle with velocity but no force should move in a straight line
        let mut particle = TestParticle::new(1.0)
            .at(DVec3::ZERO)
            .with_velocity(DVec3::new(1.0, 0.0, 0.0));

        let dt = 0.1;

        // Position step (no force, so accel = 0)
        let old_accel = verlet_position_step(&mut particle, dt);
        assert_eq!(old_accel, DVec3::ZERO);

        // After dt=0.1s with v=1 m/s, should have moved 0.1m
        assert_relative_eq!(particle.position().x, 0.1, epsilon = 1e-10);

        // Complete velocity step (still no force)
        verlet_velocity_step(&mut particle, old_accel, dt);

        // Velocity should be unchanged
        assert_relative_eq!(particle.velocity().x, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn constant_force_gives_uniform_acceleration() {
        // F = ma, constant force should give constant acceleration
        let mass = 2.0;
        let force = DVec3::new(4.0, 0.0, 0.0); // a = F/m = 2 m/s²

        let mut particle = TestParticle::new(mass)
            .at(DVec3::ZERO)
            .with_velocity(DVec3::ZERO)
            .with_force(force);

        let dt = 0.1;

        // After one step with constant force
        let old_accel = verlet_position_step(&mut particle, dt);

        // x = 0.5 * a * t² = 0.5 * 2 * 0.01 = 0.01m
        assert_relative_eq!(particle.position().x, 0.01, epsilon = 1e-10);

        // Apply same force at new position
        particle.force = force;
        verlet_velocity_step(&mut particle, old_accel, dt);

        // v = a * t = 2 * 0.1 = 0.2 m/s
        assert_relative_eq!(particle.velocity().x, 0.2, epsilon = 1e-10);
    }

    #[test]
    fn harmonic_oscillator_conserves_energy() {
        // Simple harmonic oscillator: F = -kx
        // Energy should be conserved (within numerical precision)
        let mass = 1.0;
        let k = 1.0; // Spring constant
        let x0 = 1.0; // Initial displacement

        let mut particle = TestParticle::new(mass)
            .at(DVec3::new(x0, 0.0, 0.0))
            .with_velocity(DVec3::ZERO);

        // Calculate initial force
        particle.force = DVec3::new(-k * particle.position.x, 0.0, 0.0);

        // Initial energy: E = 0.5*k*x² + 0.5*m*v² = 0.5*1*1 + 0 = 0.5
        let initial_energy = 0.5 * k * x0 * x0;

        let dt = 0.001; // Small timestep for accuracy
        let steps = 10000; // Simulate for a while

        for _ in 0..steps {
            // Position step
            let old_accel = verlet_position_step(&mut particle, dt);

            // Recalculate force at new position
            particle.force = DVec3::new(-k * particle.position.x, 0.0, 0.0);

            // Velocity step
            verlet_velocity_step(&mut particle, old_accel, dt);
        }

        // Final energy
        let final_ke = kinetic_energy(&particle);
        let final_pe = 0.5 * k * particle.position.x.powi(2);
        let final_energy = final_ke + final_pe;

        // Energy should be conserved to within ~0.1%
        assert_relative_eq!(final_energy, initial_energy, max_relative = 0.001);
    }

    #[test]
    fn two_body_coulomb_circular_orbit() {
        // Electron in circular orbit around proton
        // This tests energy conservation in a Coulomb system
        use crate::physics::constants::{COULOMB_CONSTANT, ELEMENTARY_CHARGE, ELECTRON_MASS, ANGSTROM};
        use crate::physics::coulomb::coulomb_force;

        // Proton at origin (stationary for this test - mass >> electron)
        let proton_pos = DVec3::ZERO;
        let proton_charge = ELEMENTARY_CHARGE;

        // Calculate orbital velocity for circular orbit at radius r
        // Centripetal force = Coulomb force: mv²/r = ke²/r²
        // v = sqrt(ke²/(mr))
        let r = ANGSTROM;
        let orbital_velocity = (COULOMB_CONSTANT * ELEMENTARY_CHARGE.powi(2)
            / (ELECTRON_MASS * r)).sqrt();

        // Electron at (r, 0, 0) with velocity in y direction for circular orbit
        let mut electron = TestParticle::new(ELECTRON_MASS)
            .at(DVec3::new(r, 0.0, 0.0))
            .with_velocity(DVec3::new(0.0, orbital_velocity, 0.0));

        // Calculate initial force
        let force = coulomb_force(
            -ELEMENTARY_CHARGE,
            proton_charge,
            electron.position,
            proton_pos
        );
        electron.force = force;

        // Total energy: E = KE + PE = 0.5*mv² + k*q1*q2/r
        // For circular orbit: KE = 0.5*ke²/r, PE = -ke²/r, E = -0.5*ke²/r
        let initial_ke = kinetic_energy(&electron);
        let initial_pe = COULOMB_CONSTANT * (-ELEMENTARY_CHARGE) * proton_charge / r;
        let initial_energy = initial_pe + initial_ke;

        // Simulate for ~1/10 of an orbit
        // Orbital period T = 2πr/v ≈ 2π * 1e-10 / 1.6e6 ≈ 4e-16 s
        let dt = 1.0e-19; // 0.1 attosecond timestep for accuracy
        let steps = 400; // ~4e-17 s total, about 1/10 orbit

        for _ in 0..steps {
            let old_accel = verlet_position_step(&mut electron, dt);

            // Recalculate Coulomb force at new position
            let force = coulomb_force(
                -ELEMENTARY_CHARGE,
                proton_charge,
                electron.position,
                proton_pos
            );
            electron.force = force;

            verlet_velocity_step(&mut electron, old_accel, dt);
        }

        // Calculate final energy
        let final_ke = kinetic_energy(&electron);
        let final_r = electron.position.length();
        let final_pe = COULOMB_CONSTANT * (-ELEMENTARY_CHARGE) * proton_charge / final_r;
        let final_energy = final_pe + final_ke;

        // Energy should be conserved to within 1%
        assert_relative_eq!(final_energy, initial_energy, max_relative = 0.01);

        // Radius should stay approximately constant (circular orbit)
        assert_relative_eq!(final_r, r, max_relative = 0.05);
    }

    #[test]
    fn kinetic_energy_calculation() {
        let particle = TestParticle::new(2.0)
            .with_velocity(DVec3::new(3.0, 4.0, 0.0)); // |v| = 5

        // KE = 0.5 * m * v² = 0.5 * 2 * 25 = 25
        assert_relative_eq!(kinetic_energy(&particle), 25.0, epsilon = 1e-10);
    }
}
