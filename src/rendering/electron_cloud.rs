// Electron probability cloud shader/rendering
// Electrons are rendered as fuzzy, shimmering probability clouds

use bevy::prelude::*;

/// Component that marks an entity for electron cloud rendering.
#[derive(Component, Debug, Clone)]
pub struct ElectronCloudVisual {
    /// Base color of the electron cloud
    pub color: Color,
    /// Radius of the cloud at which opacity fades to near-zero (in pixels)
    pub radius: f32,
    /// Core opacity (0.0 = invisible, 1.0 = opaque at center)
    pub opacity: f32,
    /// Animation phase for shimmer effect (radians)
    pub shimmer_phase: f32,
    /// Shimmer frequency (radians per second)
    pub shimmer_frequency: f32,
}

impl Default for ElectronCloudVisual {
    fn default() -> Self {
        Self {
            // Cool blue color for negative charge
            color: Color::srgba(0.3, 0.5, 1.0, 0.6),
            radius: 100.0,  // Corresponds to ~1 Bohr radius at default scale
            opacity: 0.6,
            shimmer_phase: 0.0,
            shimmer_frequency: 2.0,  // Subtle shimmer
        }
    }
}

impl ElectronCloudVisual {
    /// Create a cloud visual with custom color
    pub fn with_color(color: Color) -> Self {
        Self { color, ..Default::default() }
    }

    /// Create a cloud visual with custom radius
    pub fn with_radius(radius: f32) -> Self {
        Self { radius, ..Default::default() }
    }

    /// Update the shimmer animation
    pub fn update_shimmer(&mut self, delta_time: f32) {
        self.shimmer_phase += self.shimmer_frequency * delta_time;
        // Keep phase in reasonable range
        if self.shimmer_phase > std::f32::consts::TAU {
            self.shimmer_phase -= std::f32::consts::TAU;
        }
    }

    /// Get the current shimmer scale factor (for pulsing effect)
    pub fn shimmer_scale(&self) -> f32 {
        // Subtle 5% size variation
        1.0 + 0.05 * self.shimmer_phase.sin()
    }

    /// Calculate opacity at a given distance from center (normalized 0-1)
    /// This follows the 1s orbital probability density falloff
    pub fn opacity_at_distance(&self, normalized_distance: f32) -> f32 {
        if normalized_distance >= 1.0 {
            return 0.0;
        }
        // Exponential falloff similar to 1s orbital
        // |ψ|² ∝ e^(-2r/a₀)
        self.opacity * (-2.0 * normalized_distance).exp()
    }
}

/// Configuration for electron cloud rendering.
#[derive(Resource, Debug, Clone)]
pub struct ElectronCloudConfig {
    /// Number of concentric rings to draw (more = smoother gradient)
    pub ring_count: u32,
    /// Base color for electrons
    pub color: Color,
    /// Whether to animate the shimmer effect
    pub animate_shimmer: bool,
}

impl Default for ElectronCloudConfig {
    fn default() -> Self {
        Self {
            ring_count: 8,
            color: Color::srgba(0.3, 0.5, 1.0, 0.6),
            animate_shimmer: true,
        }
    }
}

/// Visual state of the electron cloud based on energy/stress
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CloudState {
    /// Relaxed, ground state (cool blue)
    Relaxed,
    /// Slightly excited (warmer hue)
    Excited,
    /// High energy / stressed (hot colors)
    Stressed,
}

impl CloudState {
    /// Get the color associated with this state
    pub fn color(&self) -> Color {
        match self {
            CloudState::Relaxed => Color::srgba(0.3, 0.5, 1.0, 0.6),
            CloudState::Excited => Color::srgba(0.5, 0.4, 0.9, 0.6),
            CloudState::Stressed => Color::srgba(0.9, 0.3, 0.3, 0.6),
        }
    }

    /// Determine state from kinetic energy relative to some threshold
    pub fn from_energy_ratio(ratio: f64) -> Self {
        if ratio < 0.3 {
            CloudState::Relaxed
        } else if ratio < 0.7 {
            CloudState::Excited
        } else {
            CloudState::Stressed
        }
    }
}

/// Bundle for spawning an electron with its cloud visual.
#[derive(Bundle, Default)]
pub struct ElectronCloudBundle {
    pub visual: ElectronCloudVisual,
    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl ElectronCloudBundle {
    /// Create an electron cloud bundle at a given screen position
    pub fn at_position(x: f32, y: f32) -> Self {
        let visual = ElectronCloudVisual::default();
        Self {
            sprite: Sprite {
                color: visual.color,
                custom_size: Some(Vec2::splat(visual.radius * 2.0)),
                ..default()
            },
            transform: Transform::from_xyz(x, y, -1.0), // Behind protons
            visual,
            ..default()
        }
    }

    /// Create with a specific radius (in pixels)
    pub fn with_radius(x: f32, y: f32, radius: f32) -> Self {
        let visual = ElectronCloudVisual::with_radius(radius);
        Self {
            sprite: Sprite {
                color: visual.color,
                custom_size: Some(Vec2::splat(radius * 2.0)),
                ..default()
            },
            transform: Transform::from_xyz(x, y, -1.0),
            visual,
            ..default()
        }
    }
}

/// Calculate the visual radius for a probability cloud based on the
/// Bohr radius and a visibility threshold.
///
/// Returns the radius in pixels at which the probability density
/// falls to `threshold` of its maximum.
pub fn cloud_visual_radius(
    bohr_radius_pixels: f32,
    threshold: f32,
) -> f32 {
    // For 1s orbital: |ψ|² = C * e^(-2r/a₀)
    // Solve for r when e^(-2r/a₀) = threshold
    // -2r/a₀ = ln(threshold)
    // r = -a₀ * ln(threshold) / 2
    -bohr_radius_pixels * threshold.ln() / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn default_cloud_has_cool_color() {
        let visual = ElectronCloudVisual::default();
        let color = visual.color.to_srgba();
        // Should be bluish
        assert!(color.blue > color.red);
        assert!(color.blue > 0.8);
    }

    #[test]
    fn shimmer_scale_oscillates() {
        let mut visual = ElectronCloudVisual::default();

        // At phase 0, sin(0) = 0
        visual.shimmer_phase = 0.0;
        assert_relative_eq!(visual.shimmer_scale(), 1.0, epsilon = 0.001);

        // At phase π/2, sin = 1
        visual.shimmer_phase = std::f32::consts::FRAC_PI_2;
        assert_relative_eq!(visual.shimmer_scale(), 1.05, epsilon = 0.001);

        // At phase 3π/2, sin = -1
        visual.shimmer_phase = 3.0 * std::f32::consts::FRAC_PI_2;
        assert_relative_eq!(visual.shimmer_scale(), 0.95, epsilon = 0.001);
    }

    #[test]
    fn opacity_falloff() {
        let visual = ElectronCloudVisual::default();

        // At center, full opacity
        let center_opacity = visual.opacity_at_distance(0.0);
        assert_relative_eq!(center_opacity, visual.opacity, epsilon = 0.001);

        // At edge (normalized distance = 1), very low opacity
        let edge_opacity = visual.opacity_at_distance(1.0);
        assert_relative_eq!(edge_opacity, 0.0, epsilon = 0.001);

        // Opacity should decrease monotonically
        let mid_opacity = visual.opacity_at_distance(0.5);
        assert!(mid_opacity < center_opacity);
        assert!(mid_opacity > edge_opacity);
    }

    #[test]
    fn cloud_state_from_energy() {
        assert_eq!(CloudState::from_energy_ratio(0.0), CloudState::Relaxed);
        assert_eq!(CloudState::from_energy_ratio(0.2), CloudState::Relaxed);
        assert_eq!(CloudState::from_energy_ratio(0.5), CloudState::Excited);
        assert_eq!(CloudState::from_energy_ratio(0.8), CloudState::Stressed);
        assert_eq!(CloudState::from_energy_ratio(1.0), CloudState::Stressed);
    }

    #[test]
    fn cloud_visual_radius_calculation() {
        let bohr_radius_pixels = 52.9; // Approximate Bohr radius in our scale

        // At 1% threshold, radius should be larger
        let r_1_percent = cloud_visual_radius(bohr_radius_pixels, 0.01);
        let r_10_percent = cloud_visual_radius(bohr_radius_pixels, 0.1);

        assert!(r_1_percent > r_10_percent);
        assert!(r_1_percent > 0.0);
    }

    #[test]
    fn shimmer_update() {
        let mut visual = ElectronCloudVisual::default();
        visual.shimmer_phase = 0.0;

        visual.update_shimmer(1.0);

        // After 1 second at frequency 2.0, phase should be 2.0 radians
        assert_relative_eq!(visual.shimmer_phase, 2.0, epsilon = 0.001);
    }
}
