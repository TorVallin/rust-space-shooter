mod camera;
mod combat;
mod enemy;
mod particles;
mod plugins;
mod state;

use std::ops::Add;

use crate::plugins::enemy_wave_plugin::EnemyWavePlugin;
use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    prelude::{
        in_state, shape, App, AssetServer, Assets, BuildChildren, Camera, Camera3dBundle, Color,
        Commands, Component, DespawnRecursiveExt, Entity, EventWriter, Input, IntoSystemConfigs,
        KeyCode, Mesh, OnEnter, PbrBundle, PluginGroup, PointLight, PointLightBundle, Quat, Query,
        Res, ResMut, Resource, SpatialBundle, StandardMaterial, Startup, Transform, Update, Vec2,
        Vec3, With, Without, OnExit,
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
use bevy_hanabi::{CompiledParticleEffect, EffectAsset, EffectSpawner, HanabiPlugin};
use bevy_rapier3d::{
    prelude::{
        ActiveEvents, Collider, GravityScale, NoUserData, RapierContext, RapierPhysicsPlugin,
        RigidBody, Sensor,
    },
    render::RapierDebugRenderPlugin,
};
use camera::{on_hit_camera_shake, CameraShakeEvent, CameraState};
use combat::{Bullet, Damageable, EntityDeath, LargeHitEffect, ParticleHitEffect, SmallHitEffect};
use particles::create_effect;
use plugins::{
    enemy_wave_plugin::EnemyAIState,
    main_menu::MainMenuPlugin,
    powerups::{Powerup, PowerupComponent, PowerupPlugin},
};
use state::GameState;

#[derive(Component, Default)]
struct Player {
    lives: u32,
    bullet_cooldown: f32,
    bullet_cooldown_timer: f32,
    active_powerup: Option<Powerup>,
}

#[derive(Resource, Default)]
struct GameResources {
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
        .add_plugins(DefaultPlugins.set(RenderPlugin { wgpu_settings }))
        .add_plugins(HanabiPlugin)
        .add_plugins(bevy_obj::ObjPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins((MainMenuPlugin, EnemyWavePlugin, PowerupPlugin))
        .add_state::<GameState>()
        .init_resource::<GameResources>()
        .insert_resource(ResolutionSettings {
            standard: Vec2::new(600.0, 1000.0),
        })
        .insert_resource(EnemyAIState::default())
        .insert_resource(CameraState::default())
        .add_event::<CameraShakeEvent>()
        .add_systems(
            Startup,
            (set_resolution, setup_cameras, setup_particle_systems),
        )
        .add_systems(
            OnEnter(GameState::Game), // run if in game state
            setup_game_state,
        )
        .add_systems(OnExit(GameState::Game), destroy_entities)
        .add_systems(
            Update,
            (
                player_controls,
                bullet_controls,
                check_bullet_damage,
                create_explosion_particle_system,
                on_hit_camera_shake,
            )
                .run_if(in_state(GameState::Game)),
        )
        .run();
}

fn set_resolution(mut windows: Query<&mut Window>, resolution: Res<ResolutionSettings>) {
    let mut window = windows.single_mut();
    let resolution = resolution.standard;
    window.resolution.set(resolution.x, resolution.y);
}

fn setup_cameras(mut commands: Commands, _: ResMut<GameResources>, camera_state: Res<CameraState>) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..Default::default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            transform: Transform::from_translation(camera_state.original_position)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..Default::default()
        },
        BloomSettings::default(),
    ));
}

fn setup_game_state(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game: ResMut<GameResources>,
) {
    game.score = 0;
    if let Some(player) = game.player {
        commands.entity(player).despawn_recursive();
    }

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

fn destroy_entities(mut commands: Commands, query: Query<Entity, With<Bullet>>) {
    for bullet in query.iter() {
        commands.entity(bullet).despawn_recursive();
    }
}

fn setup_particle_systems(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    create_effect("death_effect", 1000., true, &mut effects, &mut commands);
    create_effect("hit_effect", 50., false, &mut effects, &mut commands);
}

fn player_controls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    input: Res<Input<KeyCode>>,
    game: ResMut<GameResources>,
    mut player_query: Query<(&mut Transform, &mut Player, Option<&PowerupComponent>)>,
    time: Res<Time>,
) {
    if game.player.is_none() {
        return;
    }
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
    mut ev: EventWriter<CameraShakeEvent>,
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
                damageable.health = damageable.health.checked_sub(bullet.damage).unwrap_or(0);
                let mut intensity = 0.5;
                let mut entity_died = false;

                // Prevent the player from damaging itself & enemies from damaging eachother
                if damageable.is_player != bullet.is_player_bullet {
                    commands.entity(bullet_entity).despawn_recursive();
                    if damageable.health == 0 {
                        commands.entity(damageable_entity).despawn_recursive();

                        // Spawn a particle system as a death effect
                        commands.spawn(EntityDeath {
                            position: position.translation,
                            is_player: damageable.is_player,
                        });

                        intensity = 1.0;
                        entity_died = true;
                    }
                }

                ev.send(CameraShakeEvent { intensity });
                commands.spawn(ParticleHitEffect {
                    position: position.translation,
                    is_large: entity_died,
                });
            }
        }
    }
}

fn bullet_controls(
    _: ResMut<GameResources>,
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
    mut small_effect: Query<
        (
            &mut CompiledParticleEffect,
            &mut EffectSpawner,
            &mut Transform,
        ),
        (With<SmallHitEffect>, Without<LargeHitEffect>),
    >,
    mut large_effect: Query<
        (
            &mut CompiledParticleEffect,
            &mut EffectSpawner,
            &mut Transform,
        ),
        (With<LargeHitEffect>, Without<SmallHitEffect>),
    >,
    particle_effects: Query<(Entity, &ParticleHitEffect)>,
) {
    // TODO: Refactor this, ideally we should just be able to change the rate of the spawner
    // so that we have a single spawner. That way, we can avoid tagging with SmallHitEffect and LargeHitEffect.
    let Ok((_, mut small_spawner, mut small_transform)) = small_effect.get_single_mut() else {
        return;
    };
    let Ok((_, mut large_spawner, mut large_transform)) = large_effect.get_single_mut() else {
        return;
    };

    for (entity, particle_effect) in particle_effects.iter() {
        if particle_effect.is_large {
            large_transform.translation = particle_effect.position;
            large_spawner.reset();
        } else {
            small_transform.translation = particle_effect.position;
            small_spawner.reset();
        }
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
