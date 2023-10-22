mod combat;
mod enemy;
mod plugins;

use std::ops::Add;

use crate::plugins::enemy_wave_plugin::EnemyWavePlugin;
use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    prelude::{
        shape, App, AssetServer, Assets, BuildChildren, Camera, Camera3dBundle, Color, Commands,
        Component, DespawnRecursiveExt, Entity, Input, KeyCode, Mesh, Name, PbrBundle, PluginGroup,
        PointLight, PointLightBundle, Quat, Query, Res, ResMut, Resource, SpatialBundle,
        StandardMaterial, Startup, Transform, Update, Vec2, Vec3, Vec4, With,
    },
    render::{
        settings::{WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
    scene::SceneBundle,
    time::Time,
    transform::TransformBundle,
    window::Window,
    DefaultPlugins,
};
use bevy_hanabi::{
    Attribute, ColorOverLifetimeModifier, CompiledParticleEffect, EffectAsset, EffectSpawner,
    ExprWriter, Gradient, HanabiPlugin, LinearDragModifier, ParticleEffectBundle, ScalarType,
    SetAttributeModifier, SetPositionSphereModifier, SetVelocitySphereModifier, ShapeDimension,
    SizeOverLifetimeModifier, Spawner,
};
use bevy_rapier3d::{
    prelude::{
        ActiveEvents, Collider, GravityScale, NoUserData, RapierContext, RapierPhysicsPlugin,
        RigidBody, Sensor,
    },
    render::RapierDebugRenderPlugin,
};
use combat::{Bullet, Damageable, DeathEffect};
use plugins::{
    enemy_wave_plugin::EnemyAIState,
    powerups::{Powerup, PowerupComponent, PowerupPlugin},
};

#[derive(Component, Default)]
struct Player {
    lives: u32,
    bullet_cooldown: f32,
    bullet_cooldown_timer: f32,
    active_powerup: Option<Powerup>,
}

#[derive(Resource, Default)]
struct GameState {
    player: Option<Entity>,
    score: u32,
}

#[derive(Resource)]
struct ResolutionSettings {
    standard: Vec2,
}

fn main() {
    let mut wgpu_settings = WgpuSettings::default();
    wgpu_settings
        .features
        .set(WgpuFeatures::VERTEX_WRITABLE_STORAGE, true);

    App::new()
        .insert_resource(ResolutionSettings {
            standard: Vec2::new(600.0, 1000.0),
        })
        .insert_resource(EnemyAIState::default())
        .add_plugins(DefaultPlugins.set(RenderPlugin { wgpu_settings }))
        .add_plugins(HanabiPlugin)
        .add_plugins(bevy_obj::ObjPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins((EnemyWavePlugin, PowerupPlugin))
        .init_resource::<GameState>()
        .add_systems(Startup, set_resolution)
        .add_systems(
            Startup,
            (setup_cameras, setup_game_state, setup_particle_systems),
        )
        .add_systems(Update, (player_controls, bullet_controls))
        .add_systems(Update, check_bullet_damage)
        .add_systems(Update, create_explosion_particle_system)
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
            transform: Transform::from_xyz(0.0, 20.0, 2.0)
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
                    translation: Vec3::new(0.0, 0.0, 7.0),
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
                lives: 3,
                bullet_cooldown: 0.0,
                bullet_cooldown_timer: 0.25,
                active_powerup: None,
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

fn setup_particle_systems(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let spawner = Spawner::once(1000.0.into(), false);

    let writer = ExprWriter::new();

    let age = writer.lit(0.).uniform(writer.lit(0.2)).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);
    let lifetime = writer.lit(0.25).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let drag = writer.lit(2.).expr();
    let update_drag = LinearDragModifier::new(drag);

    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient1.add_key(0.1, Vec4::new(4.0, 4.0, 0.0, 1.0));
    color_gradient1.add_key(0.9, Vec4::new(4.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(4.0, 0.0, 0.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(0.1));
    size_gradient1.add_key(0.3, Vec2::splat(0.1));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.5).expr(),
        dimension: ShapeDimension::Volume,
    };

    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: (writer.rand(ScalarType::Float) * writer.lit(5.0) + writer.lit(10.0)).expr(),
    };

    let effect = effects.add(
        EffectAsset::new(32768, spawner, writer.finish())
            .with_name("death_effect")
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .update(update_drag)
            .render(ColorOverLifetimeModifier {
                gradient: color_gradient1,
            })
            .render(SizeOverLifetimeModifier {
                gradient: size_gradient1,
                screen_space_size: false,
            }),
    );

    commands
        .spawn(ParticleEffectBundle::new(effect).with_spawner(spawner))
        .insert(Name::new("effect"));
}

fn player_controls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    input: Res<Input<KeyCode>>,
    game: ResMut<GameState>,
    mut player_query: Query<(&mut Transform, &mut Player, Option<&PowerupComponent>)>,
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
        let mut spawn_positions = Vec::new();
        spawn_positions.push(Vec3::new(0.0, 0.0, -0.5));
        if let Some(powerup) = player.2 {
            match powerup.powerup {
                Powerup::DoubleShot => {
                    spawn_positions.push(Vec3::new(-0.2, 0.0, 0.0));
                }
                Powerup::TripleShot => {
                    spawn_positions.push(Vec3::new(-0.2, 0.0, 0.0));
                    spawn_positions.push(Vec3::new(0.2, 0.0, 0.0));
                }
            }
        }
        for pos in spawn_positions {
            spawn_bullet(
                &mut commands,
                &mut meshes,
                &mut materials,
                translation.add(pos),
            );
        }
    }
}

fn check_bullet_damage(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    mut damageables: Query<
        (Entity, &mut Damageable, &Transform),
        (With<Collider>, With<Damageable>),
    >,
    bullets: Query<(Entity, &Bullet), With<Collider>>,
) {
    // TODO: Consider doing the deletion, spawning particle effects, etc. in another system

    for (damageable_entity, mut damageable, position) in damageables.iter_mut() {
        for (bullet_entity, bullet) in &bullets {
            // Check what the bullets are hitting

            // Checks for intersections between Damageable things and the bullets
            if rapier_context.intersection_pair(damageable_entity, bullet_entity) == Some(true) {
                damageable.health -= bullet.damage;

                // Prevent the player from damaging itself & enemies from damaging eachother
                if damageable.is_player != bullet.is_player_bullet {
                    commands.entity(bullet_entity).despawn_recursive();
                    if damageable.health <= 0 {
                        commands.entity(damageable_entity).despawn_recursive();

                        // Spawn a particle system as a death effect
                        commands.spawn(DeathEffect {
                            position: position.translation,
                            is_player: damageable.is_player,
                        });
                    }
                }
            }
        }
    }
}

fn bullet_controls(
    _: ResMut<GameState>,
    mut bullets: Query<(&mut Transform, &Bullet), With<Collider>>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();
    for (mut transform, bullet) in bullets.iter_mut() {
        let direction = if bullet.up_direction { -1.0 } else { 1.0 };
        transform.translation.z += direction * bullet.velocity * delta_time;
    }
}

fn create_explosion_particle_system(
    mut commands: Commands,
    mut effect: Query<(
        &mut CompiledParticleEffect,
        &mut EffectSpawner,
        &mut Transform,
    )>,
    particle_effects: Query<(Entity, &DeathEffect)>,
) {
    let Ok((_, mut spawner, mut effect_transform)) = effect.get_single_mut() else {
        return;
    };

    for (entity, particle_effect) in particle_effects.iter() {
        effect_transform.translation = particle_effect.position;

        spawner.reset();
        commands.entity(entity).despawn();
    }
}

fn spawn_bullet(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    translation: Vec3,
) {
    commands
        .spawn(SpatialBundle::default())
        .insert(Collider::cuboid(0.05, 0.05, 0.1))
        .insert(Sensor)
        .insert(Bullet {
            is_player_bullet: true,
            up_direction: true,
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
