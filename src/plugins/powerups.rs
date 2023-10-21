use bevy::{
    prelude::{
        shape, Assets, BuildChildren, Color, Commands, Component, Mesh, PbrBundle, Plugin, Quat,
        Query, Res, ResMut, SpatialBundle, StandardMaterial, Transform, Update, Vec3, With,
    },
    time::Time,
    transform::TransformBundle,
};
use bevy_rapier3d::prelude::{ActiveEvents, Collider, RigidBody, Sensor};
use rand::Rng;

use crate::combat::DeathEffect;

struct DoubleShot {}
struct TripleShot {}

pub enum Powerup {
    DoubleShot,
    TripleShot,
}

#[derive(Component)]
pub struct PowerupComponent {
    powerup: Powerup,
}

#[derive(Component)]
pub struct PowerupTimer {
    time_left: f32,
}

pub struct PowerupPlugin;

impl Plugin for PowerupPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (
                spawn_powerups,
                update_powerup_positions,
                detect_powerup_collisions,
            ),
        );
    }
}

fn spawn_powerups(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    deaths: Query<&DeathEffect>,
) {
    let mut rng = rand::thread_rng();
    for death in deaths.iter() {
        if death.is_player {
            continue;
        }

        let prob = rng.gen::<f64>();
        if true || prob < 0.1 {
            // Spawn a new powerup
            commands
                .spawn(SpatialBundle::default())
                .insert(PowerupComponent {
                    powerup: Powerup::DoubleShot,
                })
                .insert(Collider::cuboid(0.05, 0.05, 0.1))
                .insert(RigidBody::Fixed)
                .insert(Sensor)
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(TransformBundle::from(Transform::from_translation(
                    death.position,
                )))
                .with_children(|children| {
                    children.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Capsule {
                            radius: 0.05,
                            depth: 0.10,
                            ..Default::default()
                        })),
                        transform: Transform::from_rotation(Quat::from_rotation_x(
                            -90.0f32.to_radians(),
                        )),
                        material: materials.add(StandardMaterial {
                            emissive: Color::rgb_linear(1.0, 35.0, 2.0),
                            ..Default::default()
                        }),
                        ..Default::default()
                    });
                });
        }
    }
}

// TODO: Refactor s.t. the bullets use the same component that controls movement (e.g. VerticalMovementComponent)
fn update_powerup_positions(
    mut powerups: Query<&mut Transform, (With<Collider>, With<PowerupComponent>)>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();
    for mut transform in powerups.iter_mut() {
        transform.translation.z += 1.1 * delta_time;
        transform.rotate_y(1.5 * time.delta_seconds());
        transform.rotate_x(1.75 * time.delta_seconds());
    }
}

// Sees if the player collides with a powerup
fn detect_powerup_collisions(mut commands: Commands) {}
