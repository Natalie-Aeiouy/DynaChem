# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Dynachem is an open-source educational chemistry app that teaches chemistry (especially organic chemistry) through tactile, interactive simulations. The goal is to make learning feel like play, not homework.

**Target Platform**: Mobile-first (touch input), with eventual web (WASM) deployment.

**Tech Stack**: Rust with Bevy game engine, targeting 60fps for smooth tactile feedback.

## Core Design Philosophy

### Resolution, Not Revision
Never teach a model that must be "unlearned" later. Atoms are always rendered as fuzzy probability clouds (metaballs), not hard spheres. As the player progresses, the "resolution" increases to reveal more detail (dipoles, orbital lobes) without contradicting earlier understanding.

### Embodied Learning
The player physically performs chemistry concepts through touch/drag mechanics rather than clicking buttons. If making a bond, the player drags atoms together and feels resistance (Coulomb repulsion) until they snap together.

### Stealth Learning
Game mechanics should teach concepts implicitly. The player thinks they're playing a game; they're actually learning chemistry. Equations come after the player has already experienced the concept tactilely.

## Architecture Concepts

### The Two Layers

1. **Nano-Forge** (Sandbox minigames): First-person atomic-scale manipulation using Lennard-Jones physics. Player hand-assembles individual molecules.

2. **Macro-Lab** (Overworld): Top-down/isometric view for managing reactions at scale. Unlocked machines automate bulk synthesis.

### Key Mechanics to Implement

- **Lennard-Jones Potential**: Core physics for atom interactions (attraction at mid-range, repulsion when too close, equilibrium "snap")
- **Elastic Input**: Atoms connect to cursor via virtual spring (Hooke's Law) - atoms never teleport to cursor position
- **Thermostat Slider**: Global energy control affecting molecular motion and bond stability
- **Metaball Rendering**: Atoms rendered as squishy blobs that merge when bonding

### Progression System
- Periodic table as skill tree (unlock elements progressively)
- Metroidvania-style tool unlocks (new visualization modes, manipulation tools)
- Cheat code (`BIGBANG`) unlocks all tools for advanced users

## Visual Feedback Standards

- **Relaxed state**: Cool colors (blue/green)
- **High energy/stress**: Hot colors (red/white flash)
- **Bond formation**: Soft "thump" sound + particle ripple
- **Bond breaking**: Crisp "snap" sound + screen shake for high-energy breaks

## Mascot: Dyna the Velociraptor
- Communicates through state changes and animations, not text
- Reacts to player actions (nervous when molecule is unstable, shivers when cold)
- Hints given by pointing, not explaining

## Build and Test Commands

```bash
# Run all tests
cargo test --lib

# Run a specific test
cargo test --lib test_name

# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Run the app (requires system dependencies: libasound2-dev libudev-dev on Linux)
cargo run
```

## Development Methodology
Test-driven development (TDD): red phase (write failing tests), green phase (pass tests), blue phase (refactor). Target 100% test coverage for unit and integration tests.

## Code Structure

```
src/
├── physics/           # Physics simulation
│   ├── constants.rs   # SI physical constants (CODATA values)
│   ├── coulomb.rs     # Electrostatic force calculations
│   └── simulation.rs  # Velocity Verlet time integration
├── particles/         # Particle types
│   ├── proton.rs      # Proton component (+e charge)
│   └── electron.rs    # Electron + probability cloud
├── input/             # Input handling
│   └── spring.rs      # Virtual spring for elastic drag
└── rendering/         # Visual representation
    ├── proton.rs      # Proton rendering
    └── electron_cloud.rs # Probability cloud shader
```

## Physics Notes

- All physics use SI units internally (meters, seconds, Coulombs, etc.)
- Coulomb force: `F = k * q1 * q2 / r²` where k = 8.99×10⁹ N⋅m²/C²
- Simulation uses Velocity Verlet integration (symplectic, energy-conserving)
- Time scale: 1 femtosecond timestep, scaled for visibility at 60fps
- Particles implement `Integratable` trait for generic physics integration
