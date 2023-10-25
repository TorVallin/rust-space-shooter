use std::ops::{Mul, Sub};

use bevy::{
    prelude::{
        default, in_state, AssetServer, Assets, BuildChildren, Color, Commands, Component,
        DespawnRecursiveExt, Entity, Event, EventReader, EventWriter, IntoSystemConfigs, Mesh,
        NextState, NodeBundle, OnEnter, OnExit, Plugin, Query, Res, ResMut, Resource,
        SpatialBundle, StandardMaterial, Startup, TextBundle, Transform, Update, Vec3, With,
        Without,
    },
    scene::SceneBundle,
    text::{Text, TextStyle},
    time::Time,
    ui::{Style, UiRect, Val},
};
use bevy_rapier3d::prelude::{ActiveEvents, Collider, GravityScale, RigidBody, Sensor, Velocity};
use rand::Rng;

use crate::{
    combat::{spawn_bullet, Damageable},
    enemy::Enemy,
    state::GameState,
};

const ENEMY_COOLDOWN_RANGE_S: (f32, f32) = (2.0, 3.0);
const ENEMY_FIRE_PROBABILITY: f32 = 0.5;
const ENEMY_MOVE_DURATION_S: f32 = 2.0;
const ENEMY_MOVE_VELOCITY: f32 = 0.75;

pub struct EnemyWavePlugin;

pub struct Wave {
    enemies: Vec<EnemyInstance>,
}

#[derive(Event)]
pub struct NewWaveEvent {
    wave: u32,
}

#[derive(Resource)]
pub struct EnemyAIState {
    pub current_wave: u32,
    pub move_timer: f32,
    pub moving_left: bool,
}

struct EnemyInstance {
    // Positions are given in a 2D grid, where (0, 0) is in the center of the screen
    position: [i32; 2],
    ship_type: EnemyType,
    health: u32,
}

#[derive(Component)]
struct MoveToTarget {
    target: Vec3,
}

#[derive(Component)]
struct WaveUI {}

#[derive(Component)]
struct RootWaveUI {}

enum EnemyType {
    Type1,
    Type2,
    Type3,
}

impl Plugin for EnemyWavePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<NewWaveEvent>()
            .add_systems(OnEnter(GameState::Game), (init_enemy_waves, init_ui))
            .add_systems(
                OnExit(GameState::Game),
                (destroy_ui, destroy_enemies, reset_ai_state),
            )
            .add_systems(
                Update,
                (
                    update_enemies,
                    update_move_to_target,
                    change_wave,
                    update_ui,
                )
                    .run_if(in_state(GameState::Game)),
            );
    }
}

fn init_enemy_waves(
    commands: Commands,
    mut ev: EventWriter<NewWaveEvent>,
    asset_server: Res<AssetServer>,
    ai_state: Res<EnemyAIState>,
) {
    ev.send(NewWaveEvent {
        wave: ai_state.current_wave,
    });
    spawn_wave(0, commands, asset_server);
}

fn init_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style { ..default() },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(
                    TextBundle::from_section(
                        "Wave: ",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 45.0,
                            color: Color::WHITE,
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::all(Val::Px(10.)),
                        ..default()
                    }),
                )
                .insert(WaveUI {});
        })
        .insert(RootWaveUI {});
}

fn destroy_ui(mut commands: Commands, root_query: Query<Entity, With<RootWaveUI>>) {
    for ui in root_query.iter() {
        commands.entity(ui).despawn_recursive();
    }
}

fn destroy_enemies(mut commands: Commands, enemies: Query<Entity, With<Enemy>>) {
    for enemy in enemies.iter() {
        commands.entity(enemy).despawn_recursive();
    }
}

fn reset_ai_state(mut state: ResMut<EnemyAIState>) {
    *state = EnemyAIState::default();
}

fn spawn_wave(wave_id: usize, mut commands: Commands, asset_server: Res<AssetServer>) {
    let waves = get_waves();
    let wave = waves.get(wave_id).unwrap();

    let z_starting_pos_offset = -3.0;
    let x_spacing = 0.5;
    let z_spacing = 1.0;

    let mut rng = rand::thread_rng();

    for enemy in wave.enemies.iter() {
        commands
            .spawn(Enemy {
                shot_cooldown_timer: rng
                    .gen_range(ENEMY_COOLDOWN_RANGE_S.0..=ENEMY_COOLDOWN_RANGE_S.1),
            })
            .insert(Velocity::default())
            .insert(SpatialBundle {
                transform: Transform::from_translation(Vec3::new(
                    enemy.position[0] as f32 * x_spacing + rng.gen_range(-7.0..7.0),
                    0.0,
                    enemy.position[1] as f32 * z_spacing
                        + z_starting_pos_offset
                        + rng.gen_range(-7.0..-1.0),
                )),
                ..Default::default()
            })
            .insert(Damageable {
                health: enemy.health,
                is_player: false,
            })
            .insert(MoveToTarget {
                target: Vec3::new(
                    enemy.position[0] as f32 * x_spacing,
                    0.,
                    enemy.position[1] as f32 * z_spacing + z_starting_pos_offset,
                ),
            })
            .insert(RigidBody::Dynamic)
            .insert(Sensor {})
            .insert(GravityScale(0.0))
            .insert(Collider::cylinder(0.25, 0.3))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .with_children(|children| {
                children.spawn(SceneBundle {
                    transform: Transform {
                        scale: Vec3::new(0.001, 0.001, 0.001),
                        ..Default::default()
                    },
                    scene: asset_server.load(enemy.ship_type.get_ship_path()),
                    ..Default::default()
                });
            });
    }
}

fn update_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ai_state: ResMut<EnemyAIState>,
    time: Res<Time>,
    mut enemies: Query<(&mut Enemy, &mut Velocity, &Transform), Without<MoveToTarget>>,
    move_to_target: Query<Entity, With<MoveToTarget>>,
) {
    // Ensure all (non-dead) enemies have finished moving to the target position before
    // initiating left/right movement
    if !move_to_target.is_empty() {
        return;
    }
    ai_state.move_timer -= time.delta_seconds();
    if ai_state.move_timer <= 0.0 {
        // Swap direction
        ai_state.moving_left = !ai_state.moving_left;
        ai_state.move_timer = ENEMY_MOVE_DURATION_S;
    }

    let mut rng = rand::thread_rng();
    for (mut enemy, mut enemy_vel, transform) in enemies.iter_mut() {
        enemy_vel.linvel.x = if ai_state.moving_left {
            -ENEMY_MOVE_VELOCITY
        } else {
            ENEMY_MOVE_VELOCITY
        };

        // Fire with a certain probability, otherwise skip the turn and just wait for the cooldown again
        enemy.shot_cooldown_timer -= time.delta_seconds();
        if enemy.shot_cooldown_timer <= 0.0 {
            if rng.gen::<f32>() < ENEMY_FIRE_PROBABILITY {
                // Fire!
                spawn_bullet(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    transform.translation,
                    false,
                );
            }

            enemy.shot_cooldown_timer =
                rng.gen_range(ENEMY_COOLDOWN_RANGE_S.0..=ENEMY_COOLDOWN_RANGE_S.1);
        }
    }
}

fn update_move_to_target(
    mut commands: Commands,
    mut enemies: Query<(Entity, &MoveToTarget, &mut Velocity, &mut Transform), With<Enemy>>,
) {
    for (enemy_entity, target, mut enemy_vel, mut transform) in enemies.iter_mut() {
        if target.target.distance(transform.translation) > 0.10 {
            let new_direction = target
                .target
                .sub(transform.translation)
                .normalize()
                .mul(ENEMY_MOVE_VELOCITY * 10.);
            enemy_vel.linvel = new_direction;
        } else {
            enemy_vel.linvel = Vec3::ZERO;
            transform.translation = target.target;
            commands.entity(enemy_entity).remove::<MoveToTarget>();
        }
    }
}

fn change_wave(
    commands: Commands,
    mut ev: EventWriter<NewWaveEvent>,
    asset_server: Res<AssetServer>,
    mut ai_state: ResMut<EnemyAIState>,
    mut next_state: ResMut<NextState<GameState>>,
    enemies: Query<With<Enemy>>,
) {
    if !enemies.is_empty() {
        return;
    }

    let waves = get_waves();
    ai_state.current_wave += 1;
    ev.send(NewWaveEvent {
        wave: ai_state.current_wave,
    });
    if ai_state.current_wave >= waves.len() as u32 {
        println!("Done with all waves!");
        next_state.set(GameState::Menu);
        return;
    }

    spawn_wave(ai_state.current_wave as usize, commands, asset_server);
}

fn update_ui(
    mut er: EventReader<NewWaveEvent>,
    asset_server: Res<AssetServer>,
    mut ui: Query<&mut Text, With<WaveUI>>,
) {
    for event in er.iter() {
        for mut text in ui.iter_mut() {
            *text = Text::from_section(
                format!("Wave: {}", event.wave),
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 45.0,
                    color: Color::WHITE,
                },
            );
        }
    }
}

impl EnemyType {
    fn get_ship_path(&self) -> String {
        match self {
            EnemyType::Type1 => "Spaceship1/model.obj".to_string(),
            EnemyType::Type2 => "Spaceship2/model.obj".to_string(),
            EnemyType::Type3 => "Spaceship3/model.obj".to_string(),
        }
    }
}

impl Default for EnemyAIState {
    fn default() -> Self {
        Self {
            current_wave: 0,
            move_timer: ENEMY_MOVE_DURATION_S / 2.0,
            moving_left: true,
        }
    }
}

// TODO: Specify this in e.g. a JSON file later?
fn get_waves() -> Vec<Wave> {
    let mut enemies0 = Vec::new();
    for col in (-4..=4).step_by(2) {
        for row in -1..=1 {
            enemies0.push(EnemyInstance {
                position: [col, row],
                ship_type: EnemyType::Type1,
                health: 2,
            });
        }
    }

    let mut enemies1 = Vec::new();
    for col in (-5..=5).step_by(2) {
        for row in -2..=2 {
            enemies1.push(EnemyInstance {
                position: [col, row],
                ship_type: EnemyType::Type2,
                health: 2,
            });
        }
    }

    let waves: Vec<Wave> = vec![Wave { enemies: enemies0 }, Wave { enemies: enemies1 }];
    waves
}
