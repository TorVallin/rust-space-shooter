use bevy::{
    prelude::{
        shape, Assets, BuildChildren, Color, Commands, Component, Mesh, PbrBundle, Quat, ResMut,
        SpatialBundle, StandardMaterial, Transform, Vec3,
    },
    transform::TransformBundle,
};
use bevy_rapier3d::prelude::{ActiveEvents, Collider, Sensor};

#[derive(Component)]
pub struct EntityDeath {
    pub position: Vec3,  // Where the death occured
    pub is_player: bool, // If it was the player (true) or an enemy that died (false)
}

#[derive(Component)]
pub struct ParticleHitEffect {
    pub position: Vec3, // Where the hit occured
    pub is_large: bool,
}

#[derive(Component)]
pub struct SmallHitEffect {}

#[derive(Component)]
pub struct LargeHitEffect {}

#[derive(Component)]
pub struct Damageable {
    pub health: u32,
    pub is_player: bool,
}

#[derive(Component)]
pub struct Bullet {
    pub is_player_bullet: bool,
    pub up_direction: bool,
    pub velocity: f32,
    pub damage: u32,
}

pub fn spawn_bullet(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    translation: Vec3,
    is_player_bullet: bool,
) {
    commands
        .spawn(SpatialBundle::default())
        .insert(Collider::cuboid(0.05, 0.05, 0.1))
        .insert(Sensor)
        .insert(Bullet {
            is_player_bullet,
            up_direction: is_player_bullet,
            velocity: 7.5,
            damage: 1,
        })
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(TransformBundle::from(Transform::from_translation(
            translation,
        )))
        .with_children(|children| {
            children.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Capsule {
                    radius: 0.05,
                    depth: 0.10,
                    ..Default::default()
                })),
                transform: Transform::from_rotation(Quat::from_rotation_x(-90.0f32.to_radians())),
                material: materials.add(StandardMaterial {
                    emissive: Color::rgb_linear(35.0, 1.0, 2.0),
                    ..Default::default()
                }),
                ..Default::default()
            });
        });
}
