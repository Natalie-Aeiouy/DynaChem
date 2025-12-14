// Physical constants in SI units
// Values from CODATA 2018 recommended values

/// Elementary charge (Coulombs)
/// The magnitude of electric charge carried by a single proton or electron
pub const ELEMENTARY_CHARGE: f64 = 1.602_176_634e-19;

/// Vacuum permittivity (Farads per meter)
/// Also known as the electric constant or permittivity of free space
pub const VACUUM_PERMITTIVITY: f64 = 8.854_187_8128e-12;

/// Coulomb constant (N⋅m²/C²)
/// k = 1 / (4πε₀)
pub const COULOMB_CONSTANT: f64 = 8.987_551_792_3e9;

/// Electron mass (kilograms)
pub const ELECTRON_MASS: f64 = 9.109_383_7015e-31;

/// Proton mass (kilograms)
pub const PROTON_MASS: f64 = 1.672_621_923_69e-27;

/// Bohr radius (meters)
/// The most probable distance between the nucleus and electron in a hydrogen atom
pub const BOHR_RADIUS: f64 = 5.291_772_109_03e-11;

/// Planck constant (Joule-seconds)
pub const PLANCK_CONSTANT: f64 = 6.626_070_15e-34;

/// Reduced Planck constant ℏ = h/(2π) (Joule-seconds)
pub const HBAR: f64 = 1.054_571_817e-34;

/// Speed of light in vacuum (meters per second)
pub const SPEED_OF_LIGHT: f64 = 299_792_458.0;

/// One Ångström in meters (convenient for atomic scales)
pub const ANGSTROM: f64 = 1.0e-10;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coulomb_constant_derived_correctly() {
        // k = 1 / (4πε₀)
        let derived_k = 1.0 / (4.0 * std::f64::consts::PI * VACUUM_PERMITTIVITY);
        let relative_error = (derived_k - COULOMB_CONSTANT).abs() / COULOMB_CONSTANT;
        assert!(relative_error < 1e-9, "Coulomb constant derivation error: {}", relative_error);
    }

    #[test]
    fn hbar_derived_correctly() {
        // ℏ = h / (2π)
        let derived_hbar = PLANCK_CONSTANT / (2.0 * std::f64::consts::PI);
        let relative_error = (derived_hbar - HBAR).abs() / HBAR;
        assert!(relative_error < 1e-9, "ℏ derivation error: {}", relative_error);
    }
}
