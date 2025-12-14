// Virtual spring for elastic touch input
// The key UX principle: atoms don't teleport to your finger.
// Your finger is connected to the atom via a spring, providing tactile feedback.

use bevy::prelude::*;
use glam::DVec3;

/// Represents an active touch/drag input in the simulation.
/// When a particle is selected, a virtual spring connects it to the cursor.
#[derive(Resource, Debug, Clone, Default)]
pub struct TouchInput {
    /// Whether input is currently active (finger down / mouse pressed)
    pub active: bool,
    /// Current input position in world coordinates (meters)
    pub position: DVec3,
    /// Entity being dragged, if any
    pub selected_entity: Option<Entity>,
}

impl TouchInput {
    /// Start a new drag interaction
    pub fn begin(&mut self, position: DVec3, entity: Entity) {
        self.active = true;
        self.position = position;
        self.selected_entity = Some(entity);
    }

    /// Update the input position during drag
    pub fn update_position(&mut self, position: DVec3) {
        self.position = position;
    }

    /// End the drag interaction
    pub fn end(&mut self) {
        self.active = false;
        self.selected_entity = None;
    }
}

/// Component that marks an entity as draggable via spring connection.
#[derive(Component, Debug, Clone)]
pub struct Draggable {
    /// Whether this entity is currently being dragged
    pub is_dragging: bool,
}

impl Default for Draggable {
    fn default() -> Self {
        Self { is_dragging: false }
    }
}

/// Configuration for the virtual spring that connects input to particles.
#[derive(Resource, Debug, Clone)]
pub struct SpringConfig {
    /// Spring constant (stiffness) in N/m
    /// Higher = more responsive, but can cause oscillation
    pub stiffness: f64,
    /// Damping coefficient in N⋅s/m
    /// Prevents oscillation when dragging
    pub damping: f64,
    /// Maximum force the spring can exert (prevents runaway)
    pub max_force: f64,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            // These values are tuned for "feels good" at atomic scales
            // Will need adjustment based on actual gameplay testing
            stiffness: 1.0e-6,  // Soft spring appropriate for atomic masses
            damping: 1.0e-12,   // Light damping
            max_force: 1.0e-6,  // Limit to prevent numerical issues
        }
    }
}

impl SpringConfig {
    /// Create a spring configuration with custom stiffness
    pub fn with_stiffness(stiffness: f64) -> Self {
        Self { stiffness, ..Default::default() }
    }
}

/// Calculate the spring force connecting a particle to the input position.
///
/// Uses Hooke's Law with damping: F = -k(x - x_target) - c*v
///
/// # Arguments
/// * `particle_pos` - Current position of the particle
/// * `particle_vel` - Current velocity of the particle
/// * `target_pos` - Position where the input (finger/cursor) is
/// * `config` - Spring configuration
///
/// # Returns
/// Force vector to apply to the particle (in Newtons)
pub fn spring_force(
    particle_pos: DVec3,
    particle_vel: DVec3,
    target_pos: DVec3,
    config: &SpringConfig,
) -> DVec3 {
    // Displacement from particle to target
    let displacement = target_pos - particle_pos;

    // Spring force: F = k * displacement (pulls toward target)
    let spring_f = config.stiffness * displacement;

    // Damping force: F = -c * velocity (opposes motion)
    let damping_f = -config.damping * particle_vel;

    // Total force
    let mut total_force = spring_f + damping_f;

    // Clamp to maximum force
    let force_magnitude = total_force.length();
    if force_magnitude > config.max_force {
        total_force = total_force.normalize() * config.max_force;
    }

    total_force
}

/// Calculate the "stretch" of the virtual spring.
/// This can be used for visual feedback (showing the spring tension).
pub fn spring_stretch(particle_pos: DVec3, target_pos: DVec3) -> f64 {
    (target_pos - particle_pos).length()
}

/// Determine the visual state of the spring based on tension.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpringState {
    /// Spring is relaxed (particle near target)
    Relaxed,
    /// Spring is slightly stretched
    Light,
    /// Spring is moderately stretched
    Medium,
    /// Spring is heavily stretched (near max force)
    Heavy,
}

impl SpringState {
    /// Determine spring state from stretch distance and config
    pub fn from_stretch(stretch: f64, config: &SpringConfig) -> Self {
        let force = stretch * config.stiffness;
        let ratio = force / config.max_force;

        if ratio < 0.1 {
            SpringState::Relaxed
        } else if ratio < 0.4 {
            SpringState::Light
        } else if ratio < 0.7 {
            SpringState::Medium
        } else {
            SpringState::Heavy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn spring_force_pulls_toward_target() {
        let config = SpringConfig::with_stiffness(1.0);
        let particle_pos = DVec3::ZERO;
        let particle_vel = DVec3::ZERO;
        let target_pos = DVec3::new(1.0, 0.0, 0.0);

        let force = spring_force(particle_pos, particle_vel, target_pos, &config);

        // Force should point toward target (positive x)
        assert!(force.x > 0.0, "Spring should pull toward target");
        assert_relative_eq!(force.y, 0.0, epsilon = 1e-10);
        assert_relative_eq!(force.z, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn spring_force_magnitude_proportional_to_distance() {
        let config = SpringConfig {
            stiffness: 2.0,
            damping: 0.0,
            max_force: 1000.0,  // High enough to not clamp
        };
        let particle_pos = DVec3::ZERO;
        let particle_vel = DVec3::ZERO;

        let target_1 = DVec3::new(1.0, 0.0, 0.0);
        let target_2 = DVec3::new(2.0, 0.0, 0.0);

        let force_1 = spring_force(particle_pos, particle_vel, target_1, &config);
        let force_2 = spring_force(particle_pos, particle_vel, target_2, &config);

        // Force at 2x distance should be 2x magnitude
        assert_relative_eq!(force_2.length(), 2.0 * force_1.length(), max_relative = 0.01);
    }

    #[test]
    fn damping_opposes_velocity() {
        let config = SpringConfig {
            stiffness: 0.0,  // No spring force
            damping: 1.0,
            max_force: 1000.0,
        };

        let particle_pos = DVec3::ZERO;
        let particle_vel = DVec3::new(1.0, 0.0, 0.0);
        let target_pos = DVec3::ZERO;

        let force = spring_force(particle_pos, particle_vel, target_pos, &config);

        // Force should oppose velocity (negative x)
        assert!(force.x < 0.0, "Damping should oppose velocity");
        assert_relative_eq!(force.x, -1.0, epsilon = 1e-10);
    }

    #[test]
    fn force_clamped_to_maximum() {
        let config = SpringConfig {
            stiffness: 100.0,  // Very stiff
            damping: 0.0,
            max_force: 1.0,    // Low max
        };

        let particle_pos = DVec3::ZERO;
        let particle_vel = DVec3::ZERO;
        let target_pos = DVec3::new(100.0, 0.0, 0.0);  // Very far

        let force = spring_force(particle_pos, particle_vel, target_pos, &config);

        // Force magnitude should be clamped to max_force
        assert_relative_eq!(force.length(), config.max_force, max_relative = 0.01);
    }

    #[test]
    fn zero_displacement_gives_zero_spring_force() {
        let config = SpringConfig::with_stiffness(1.0);
        let pos = DVec3::new(5.0, 3.0, 2.0);

        let force = spring_force(pos, DVec3::ZERO, pos, &config);

        assert_relative_eq!(force.length(), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn spring_stretch_calculation() {
        let p1 = DVec3::new(0.0, 0.0, 0.0);
        let p2 = DVec3::new(3.0, 4.0, 0.0);

        let stretch = spring_stretch(p1, p2);

        assert_relative_eq!(stretch, 5.0, epsilon = 1e-10);
    }

    #[test]
    fn spring_state_transitions() {
        let config = SpringConfig {
            stiffness: 1.0,
            damping: 0.0,
            max_force: 10.0,
        };

        // At 0 stretch, should be relaxed
        assert_eq!(SpringState::from_stretch(0.0, &config), SpringState::Relaxed);

        // At 0.5 stretch (force = 0.5, ratio = 0.05), should still be relaxed
        assert_eq!(SpringState::from_stretch(0.5, &config), SpringState::Relaxed);

        // At 2.0 stretch (force = 2.0, ratio = 0.2), should be light
        assert_eq!(SpringState::from_stretch(2.0, &config), SpringState::Light);

        // At 5.0 stretch (force = 5.0, ratio = 0.5), should be medium
        assert_eq!(SpringState::from_stretch(5.0, &config), SpringState::Medium);

        // At 8.0 stretch (force = 8.0, ratio = 0.8), should be heavy
        assert_eq!(SpringState::from_stretch(8.0, &config), SpringState::Heavy);
    }

    #[test]
    fn touch_input_lifecycle() {
        let mut input = TouchInput::default();
        let entity = Entity::from_raw(42);

        // Initially inactive
        assert!(!input.active);
        assert!(input.selected_entity.is_none());

        // Begin drag
        input.begin(DVec3::new(1.0, 2.0, 3.0), entity);
        assert!(input.active);
        assert_eq!(input.selected_entity, Some(entity));
        assert_eq!(input.position, DVec3::new(1.0, 2.0, 3.0));

        // Update position
        input.update_position(DVec3::new(4.0, 5.0, 6.0));
        assert_eq!(input.position, DVec3::new(4.0, 5.0, 6.0));

        // End drag
        input.end();
        assert!(!input.active);
        assert!(input.selected_entity.is_none());
    }
}
