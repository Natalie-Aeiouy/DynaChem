// Proton visual rendering
// Protons are rendered as small, bright points with a glow effect

use bevy::prelude::*;

/// Component that marks an entity for proton-style rendering.
/// The actual visual is created as a child sprite.
#[derive(Component, Debug, Clone)]
pub struct ProtonVisual {
    /// Base color of the proton
    pub color: Color,
    /// Size of the proton visual in pixels (at default zoom)
    pub size: f32,
    /// Glow intensity (0.0 = no glow, 1.0 = full glow)
    pub glow_intensity: f32,
}

impl Default for ProtonVisual {
    fn default() -> Self {
        Self {
            // Warm red/orange color for positive charge
            color: Color::srgb(1.0, 0.4, 0.2),
            size: 8.0,
            glow_intensity: 0.8,
        }
    }
}

impl ProtonVisual {
    /// Create a proton visual with custom color
    pub fn with_color(color: Color) -> Self {
        Self { color, ..Default::default() }
    }

    /// Create a proton visual with custom size
    pub fn with_size(size: f32) -> Self {
        Self { size, ..Default::default() }
    }
}

/// Configuration for how protons are displayed.
#[derive(Resource, Debug, Clone)]
pub struct ProtonRenderConfig {
    /// Pixels per meter (determines zoom level)
    pub scale: f64,
    /// Base color for protons
    pub color: Color,
    /// Size multiplier
    pub size_multiplier: f32,
}

impl Default for ProtonRenderConfig {
    fn default() -> Self {
        Self {
            // Default scale: 1 Ångström = 100 pixels
            // This makes atomic-scale interactions visible
            scale: 1.0e12,  // 1e-10 m (Å) * 1e12 = 100 pixels
            color: Color::srgb(1.0, 0.4, 0.2),
            size_multiplier: 1.0,
        }
    }
}

/// Bundle for spawning a proton with its visual representation.
#[derive(Bundle, Default)]
pub struct ProtonBundle {
    pub visual: ProtonVisual,
    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl ProtonBundle {
    /// Create a proton bundle at a given screen position
    pub fn at_position(x: f32, y: f32) -> Self {
        let visual = ProtonVisual::default();
        Self {
            sprite: Sprite {
                color: visual.color,
                custom_size: Some(Vec2::splat(visual.size)),
                ..default()
            },
            transform: Transform::from_xyz(x, y, 0.0),
            visual,
            ..default()
        }
    }
}

/// Convert physics position (meters) to screen position (pixels).
pub fn physics_to_screen(pos: glam::DVec3, config: &ProtonRenderConfig) -> Vec2 {
    Vec2::new(
        (pos.x * config.scale) as f32,
        (pos.y * config.scale) as f32,
    )
}

/// Convert screen position (pixels) to physics position (meters).
pub fn screen_to_physics(pos: Vec2, config: &ProtonRenderConfig) -> glam::DVec3 {
    glam::DVec3::new(
        pos.x as f64 / config.scale,
        pos.y as f64 / config.scale,
        0.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::constants::ANGSTROM;
    use approx::assert_relative_eq;

    #[test]
    fn default_visual_has_warm_color() {
        let visual = ProtonVisual::default();
        // Should be reddish/orange
        let color = visual.color.to_srgba();
        assert!(color.red > 0.8);
        assert!(color.green < 0.6);
    }

    #[test]
    fn physics_to_screen_conversion() {
        let config = ProtonRenderConfig::default();

        // 1 Ångström should convert to ~100 pixels
        let physics_pos = glam::DVec3::new(ANGSTROM, 0.0, 0.0);
        let screen_pos = physics_to_screen(physics_pos, &config);

        assert_relative_eq!(screen_pos.x, 100.0, max_relative = 0.01);
        assert_relative_eq!(screen_pos.y, 0.0, epsilon = 0.01);
    }

    #[test]
    fn screen_to_physics_conversion() {
        let config = ProtonRenderConfig::default();

        // 100 pixels should convert to ~1 Ångström
        let screen_pos = Vec2::new(100.0, 0.0);
        let physics_pos = screen_to_physics(screen_pos, &config);

        assert_relative_eq!(physics_pos.x, ANGSTROM, max_relative = 0.01);
        assert_relative_eq!(physics_pos.y, 0.0, epsilon = 1e-20);
    }

    #[test]
    fn round_trip_conversion() {
        let config = ProtonRenderConfig::default();

        let original = glam::DVec3::new(2.5e-10, -1.3e-10, 0.0);
        let screen = physics_to_screen(original, &config);
        let back = screen_to_physics(screen, &config);

        assert_relative_eq!(back.x, original.x, max_relative = 0.001);
        assert_relative_eq!(back.y, original.y, max_relative = 0.001);
    }
}
