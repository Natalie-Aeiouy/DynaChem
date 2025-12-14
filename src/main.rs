// Dynachem - Educational chemistry simulation
// Copyright (C) 2024 OntoDyn
// SPDX-License-Identifier: GPL-3.0-or-later

use bevy::prelude::*;
use glam::DVec3;

use dynachem::physics::constants::{BOHR_RADIUS, COULOMB_CONSTANT, ELEMENTARY_CHARGE};
use dynachem::physics::coulomb::coulomb_force;
use dynachem::physics::simulation::{verlet_position_step, verlet_velocity_step, Integratable};
use dynachem::particles::proton::Proton;
use dynachem::particles::electron::Electron;
use dynachem::input::spring::{spring_force, SpringConfig, TouchInput, Draggable};
use dynachem::rendering::proton::{ProtonRenderConfig, physics_to_screen, screen_to_physics};
use dynachem::rendering::electron_cloud::ElectronCloudVisual;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Dynachem - Electrostatic Playground".into(),
                resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .insert_resource(ProtonRenderConfig::default())
        .insert_resource(SpringConfig {
            stiffness: 1.0e-8,
            damping: 1.0e-15,
            max_force: 1.0e-7,
        })
        .insert_resource(TouchInput::default())
        .insert_resource(SimulationTime { dt: 1.0e-17 })
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_mouse_input,
            apply_spring_force,
            apply_coulomb_forces,
            physics_step,
            sync_visuals,
            update_electron_cloud_shimmer,
        ).chain())
        .run();
}

#[derive(Resource)]
struct SimulationTime {
    dt: f64,
}

#[derive(Component)]
struct PhysicsProton(Proton);

#[derive(Component)]
struct PhysicsElectron(Electron);

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2d);

    let render_config = ProtonRenderConfig::default();

    // Spawn proton at center, slightly offset
    let proton_physics_pos = DVec3::new(BOHR_RADIUS * 2.0, 0.0, 0.0);
    let proton_screen_pos = physics_to_screen(proton_physics_pos, &render_config);

    commands.spawn((
        PhysicsProton(Proton::new(proton_physics_pos)),
        Draggable::default(),
        Sprite {
            color: Color::srgb(1.0, 0.4, 0.2),
            custom_size: Some(Vec2::splat(16.0)),
            ..default()
        },
        Transform::from_xyz(proton_screen_pos.x, proton_screen_pos.y, 1.0),
    ));

    // Spawn electron cloud at origin
    let electron_physics_pos = DVec3::ZERO;
    let electron_screen_pos = physics_to_screen(electron_physics_pos, &render_config);

    // Calculate orbital velocity for stable orbit at 2 Bohr radii
    let r = BOHR_RADIUS * 2.0;
    let orbital_v = (COULOMB_CONSTANT * ELEMENTARY_CHARGE.powi(2)
        / (dynachem::physics::constants::ELECTRON_MASS * r)).sqrt();

    let mut electron = Electron::new(electron_physics_pos);
    // Give electron some initial velocity for interesting dynamics
    electron.velocity = DVec3::new(0.0, orbital_v * 0.5, 0.0);

    commands.spawn((
        PhysicsElectron(electron),
        ElectronCloudVisual::default(),
        Sprite {
            color: Color::srgba(0.3, 0.5, 1.0, 0.4),
            custom_size: Some(Vec2::splat(200.0)),
            ..default()
        },
        Transform::from_xyz(electron_screen_pos.x, electron_screen_pos.y, 0.0),
    ));

    // Instructions text
    commands.spawn((
        Text::new("Click and drag the orange proton!\nThe blue electron cloud responds to Coulomb forces."),
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgba(0.8, 0.8, 0.8, 0.8)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

fn handle_mouse_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut touch_input: ResMut<TouchInput>,
    render_config: Res<ProtonRenderConfig>,
    protons: Query<(Entity, &PhysicsProton, &Transform), With<Draggable>>,
) {
    let window = windows.single();
    let (camera, camera_transform) = cameras.single();

    if let Some(cursor_pos) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    {
        let physics_pos = screen_to_physics(cursor_pos, &render_config);

        if mouse_button.just_pressed(MouseButton::Left) {
            // Check if we clicked on a proton
            for (entity, proton, transform) in protons.iter() {
                let proton_screen = Vec2::new(transform.translation.x, transform.translation.y);
                if cursor_pos.distance(proton_screen) < 30.0 {
                    touch_input.begin(physics_pos, entity);
                    break;
                }
            }
        } else if mouse_button.pressed(MouseButton::Left) && touch_input.active {
            touch_input.update_position(physics_pos);
        } else if mouse_button.just_released(MouseButton::Left) {
            touch_input.end();
        }
    }
}

fn apply_spring_force(
    touch_input: Res<TouchInput>,
    spring_config: Res<SpringConfig>,
    mut protons: Query<(Entity, &mut PhysicsProton)>,
) {
    if !touch_input.active {
        return;
    }

    if let Some(selected) = touch_input.selected_entity {
        for (entity, mut proton) in protons.iter_mut() {
            if entity == selected {
                let force = spring_force(
                    proton.0.position,
                    proton.0.velocity,
                    touch_input.position,
                    &spring_config,
                );
                proton.0.apply_force(force);
            }
        }
    }
}

fn apply_coulomb_forces(
    mut protons: Query<&mut PhysicsProton>,
    mut electrons: Query<&mut PhysicsElectron>,
) {
    // Get all positions first to avoid borrow issues
    let proton_data: Vec<_> = protons.iter()
        .map(|p| (p.0.position, Proton::charge()))
        .collect();

    let electron_data: Vec<_> = electrons.iter()
        .map(|e| (e.0.position, Electron::charge()))
        .collect();

    // Apply forces from electrons to protons
    for mut proton in protons.iter_mut() {
        for (e_pos, e_charge) in &electron_data {
            let force = coulomb_force(
                Proton::charge(),
                *e_charge,
                proton.0.position,
                *e_pos,
            );
            proton.0.apply_force(force);
        }
    }

    // Apply forces from protons to electrons
    for mut electron in electrons.iter_mut() {
        for (p_pos, p_charge) in &proton_data {
            let force = coulomb_force(
                Electron::charge(),
                *p_charge,
                electron.0.position,
                *p_pos,
            );
            electron.0.apply_force(force);
        }
    }
}

fn physics_step(
    sim_time: Res<SimulationTime>,
    mut protons: Query<&mut PhysicsProton>,
    mut electrons: Query<&mut PhysicsElectron>,
) {
    let dt = sim_time.dt;

    // Multiple substeps for stability
    let substeps = 10;
    let sub_dt = dt / substeps as f64;

    for _ in 0..substeps {
        // Update protons
        for mut proton in protons.iter_mut() {
            let old_accel = verlet_position_step(&mut proton.0, sub_dt);
            verlet_velocity_step(&mut proton.0, old_accel, sub_dt);
            proton.0.clear_forces();
        }

        // Update electrons
        for mut electron in electrons.iter_mut() {
            let old_accel = verlet_position_step(&mut electron.0, sub_dt);
            verlet_velocity_step(&mut electron.0, old_accel, sub_dt);
            electron.0.clear_forces();
        }
    }
}

fn sync_visuals(
    render_config: Res<ProtonRenderConfig>,
    mut protons: Query<(&PhysicsProton, &mut Transform), Without<PhysicsElectron>>,
    mut electrons: Query<(&PhysicsElectron, &mut Transform), Without<PhysicsProton>>,
) {
    for (proton, mut transform) in protons.iter_mut() {
        let screen_pos = physics_to_screen(proton.0.position, &render_config);
        transform.translation.x = screen_pos.x;
        transform.translation.y = screen_pos.y;
    }

    for (electron, mut transform) in electrons.iter_mut() {
        let screen_pos = physics_to_screen(electron.0.position, &render_config);
        transform.translation.x = screen_pos.x;
        transform.translation.y = screen_pos.y;
    }
}

fn update_electron_cloud_shimmer(
    time: Res<Time>,
    mut clouds: Query<(&mut ElectronCloudVisual, &mut Sprite)>,
) {
    for (mut cloud, mut sprite) in clouds.iter_mut() {
        cloud.update_shimmer(time.delta_secs());

        // Pulse the size slightly
        let scale = cloud.shimmer_scale();
        sprite.custom_size = Some(Vec2::splat(200.0 * scale));

        // Subtle color shift based on phase
        let hue_shift = 0.05 * cloud.shimmer_phase.sin();
        sprite.color = Color::srgba(0.3 + hue_shift, 0.5, 1.0 - hue_shift, 0.4);
    }
}
