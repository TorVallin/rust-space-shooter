use bevy::{
    prelude::{
        shape, Assets, BuildChildren, Color, Commands, Component, DespawnRecursiveExt, Entity,
        Mesh, PbrBundle, Plugin, Quat, Query, Res, ResMut, SpatialBundle, StandardMaterial,
        Transform, Update, Vec3, With, Without,
    },
    time::Time,
    transform::TransformBundle,
};
use bevy_rapier3d::prelude::{Collider, GravityScale, RapierContext, RigidBody, Sensor, Velocity};
use rand::Rng;

use crate::{combat::DeathEffect, Player};

#[derive(Clone)]
pub enum Powerup {
    DoubleShot,
    TripleShot,
}

#[derive(Clone)]
struct DoubleShot {}
#[derive(Clone)]
struct TripleShot {}

#[derive(Component, Clone)]
pub struct PowerupComponent {
    powerup: Powerup,
    time_left: f32,
}

pub struct PowerupPlugin;

impl Plugin for PowerupPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Update,
            (update_powerups, spawn_powerups, detect_powerup_collisions),
        );
    }
}

fn update_powerups(
    mut commands: Commands,
    time: Res<Time>,
    mut powerups: Query<(Entity, &mut PowerupComponent), With<Player>>,
) {
    for (entity, mut powerup) in powerups.iter_mut() {
        powerup.time_left -= time.delta_seconds();
        if powerup.time_left < 0.0 {
            commands.entity(entity).remove::<PowerupComponent>();
        }
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
        if  prob < 0.1 {
            // Spawn a new powerup
            commands
                .spawn(SpatialBundle::default())
                .insert(PowerupComponent {
                    powerup: Powerup::DoubleShot,
                    time_left: 5.0,
                })
                .insert(RigidBody::Dynamic)
                .insert(GravityScale(0.0))
                .insert(Collider::cuboid(0.05, 0.05, 0.1))
                .insert(Sensor)
                .insert(TransformBundle::from(Transform::from_translation(
                    death.position,
                )))
                .insert(Velocity {
                    linvel: Vec3::new(0.0, 0.0, 3.0),
                    angvel: Vec3::new(1.70, 1.5, 0.0),
                })
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

// Sees if the player collides with a powerup
fn detect_powerup_collisions(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    mut player_query: Query<(Entity, &mut Player, Option<&PowerupComponent>)>,
    mut powerups: Query<(Entity, &mut PowerupComponent), (With<Collider>, Without<Player>)>,
) {
    let Ok(player) = player_query.get_single_mut() else {
        return;
    };

    for (power_entity, mut powerup) in powerups.iter_mut() {
        if rapier_context.intersection_pair(power_entity, player.0) == Some(true) {
            // Replaces the powerup the player already has, but any remaining time transfers
            if let Some(current_powerup) = player.2 {
                powerup.time_left += current_powerup.time_left;
            } else {
                commands.entity(player.0).insert(powerup.clone());
            }
            commands.entity(power_entity).despawn_recursive();
        }
    }
}
