mod combat;
mod enemy;
mod plugins;

use std::ops::Add;

use crate::plugins::enemy_wave_plugin::EnemyWavePlugin;
use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    prelude::{
        shape, App, AssetServer, Assets, BuildChildren, Camera, Camera3dBundle, Color, Commands,
        Component, Entity, Input, KeyCode, Mesh, PbrBundle, PluginGroup, PointLight,
        PointLightBundle, Quat, Query, Res, ResMut, Resource, SpatialBundle, StandardMaterial,
        Startup, Transform, Update, Vec2, Vec3, With,
    },
    scene::SceneBundle,
    time::Time,
    transform::TransformBundle,
    window::{Window, WindowResized},
    DefaultPlugins,
};
use bevy_rapier3d::{
    prelude::{
        ActiveEvents, Collider, GravityScale, NoUserData, RapierContext, RapierPhysicsPlugin,
        RigidBody, Sensor,
    },
    render::RapierDebugRenderPlugin,
};

#[derive(Component, Default)]
struct Player {
    lives: u32,
    bullet_cooldown: f32,
    bullet_cooldown_timer: f32,
}

#[derive(Resource, Default)]
struct GameState {
    player: Option<Entity>,
    score: u32,
}

#[derive(Component)]
struct EnemyBullet {}

#[derive(Component)]
struct Bullet {
    up_direction: bool,
    velocity: f32,
    damage: u32,
}

#[derive(Resource)]
struct ResolutionSettings {
    standard: Vec2,
}

fn main() {
    App::new()
        .insert_resource(ResolutionSettings {
            standard: Vec2::new(600.0, 1000.0),
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy_obj::ObjPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(EnemyWavePlugin)
        .init_resource::<GameState>()
        .add_systems(Startup, set_resolution)
        .add_systems(Startup, (setup_cameras, setup_game_state))
        .add_systems(Update, (player_controls, bullet_controls))
        .add_systems(Update, check_intersections)
        .run();
}

fn set_resolution(mut windows: Query<&mut Window>, resolution: Res<ResolutionSettings>) {
    let mut window = windows.single_mut();
    let resolution = resolution.standard;
    window.resolution.set(resolution.x, resolution.y);
}

fn setup_cameras(mut commands: Commands, _: ResMut<GameState>) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..Default::default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            transform: Transform::from_xyz(0.0, 6.0, 2.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..Default::default()
        },
        BloomSettings::default(),
    ));
}

fn setup_game_state(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game: ResMut<GameState>,
) {
    game.score = 0;

    game.player = Some(
        commands
            .spawn(SceneBundle {
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 1.5),
                    rotation: Quat::from_rotation_y(90.0_f32.to_radians()),
                    ..Default::default()
                },
                scene: asset_server.load("Spaceship4/model.obj"),
                ..Default::default()
            })
            .insert(RigidBody::Dynamic)
            .insert(GravityScale(0.0))
            .insert(Collider::cylinder(0.25, 0.3))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Player {
                lives: 0,
                bullet_cooldown: 0.0,
                bullet_cooldown_timer: 0.25,
            })
            .id(),
    );

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(3.0, 2.0, 0.0),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn player_controls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    input: Res<Input<KeyCode>>,
    game: ResMut<GameState>,
    mut player_query: Query<(&mut Transform, &mut Player)>,
    time: Res<Time>,
) {
    let player_entity = game.player.unwrap();

    let mut player = player_query.get_mut(player_entity).unwrap();
    let mut translation = player.0.translation;

    let move_speed = 3.0;
    // Move left and right with A/D
    if input.pressed(KeyCode::A) {
        translation.x -= move_speed * time.delta_seconds();
        *player.0 = Transform {
            translation,
            rotation: player.0.rotation,
            ..Default::default()
        }
    }
    if input.pressed(KeyCode::D) {
        translation.x += move_speed * time.delta_seconds();
        *player.0 = Transform {
            translation,
            rotation: player.0.rotation,
            ..Default::default()
        }
    }

    let can_shoot = if player.1.bullet_cooldown <= 0.0 {
        true
    } else {
        player.1.bullet_cooldown -= time.delta_seconds();
        false
    };

    if can_shoot && input.pressed(KeyCode::Space) {
        player.1.bullet_cooldown = player.1.bullet_cooldown_timer;
        commands
            .spawn(SpatialBundle::default())
            .insert(Collider::cuboid(0.05, 0.05, 0.1))
            .insert(RigidBody::Fixed)
            .insert(Sensor)
            .insert(Bullet {
                up_direction: true,
                velocity: 7.5,
                damage: 1,
            })
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(TransformBundle::from(Transform::from_translation(
                translation.add(Vec3::new(0.0, 0.0, -0.5)),
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
                        emissive: Color::rgb_linear(35.0, 1.0, 2.0),
                        ..Default::default()
                    }),
                    ..Default::default()
                });
            });
    }
}

fn check_intersections(
    game: ResMut<GameState>,
    rapier_context: Res<RapierContext>,
    enemy_bullets: Query<Entity, (With<Collider>, With<EnemyBullet>)>,
) {
    for bullet in &enemy_bullets {
        let player = game.player.unwrap();

        // Checks for intersections between the player and the enemy bullet
        if rapier_context.intersection_pair(player, bullet) == Some(true) {
            println!(
                "The entities {:?} and {:?} have intersecting colliders!",
                player, bullet
            );
        }
    }
}

fn bullet_controls(
    _: ResMut<GameState>,
    mut bullets: Query<(&mut Transform, &Bullet), (With<Collider>, With<Bullet>)>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();
    for (mut transform, bullet) in bullets.iter_mut() {
        let direction = if bullet.up_direction { -1.0 } else { 1.0 };
        transform.translation.z += direction * bullet.velocity * delta_time;
    }
}
