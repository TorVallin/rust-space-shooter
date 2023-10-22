use bevy::{
    prelude::{
        AssetServer, BuildChildren, Commands, Entity, Plugin, Query, Res, ResMut, Resource,
        SpatialBundle, Startup, Transform, Update, Vec3, With,
    },
    scene::SceneBundle,
    time::Time,
};
use bevy_rapier3d::prelude::{ActiveEvents, Collider, GravityScale, RigidBody, Velocity};

use crate::{combat::Damageable, enemy::Enemy};

const ENEMY_MOVE_DURATION_S: f32 = 2.0;
const ENEMY_MOVE_VELOCITY: f32 = 1.5;

pub struct EnemyWavePlugin;

pub struct Wave {
    enemies: Vec<EnemyInstance>,
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

enum EnemyType {
    Type1,
    Type2,
    Type3,
}

impl Plugin for EnemyWavePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, init_enemy_waves)
            .add_systems(Update, (update_enemies, change_wave));
    }
}

fn init_enemy_waves(commands: Commands, asset_server: Res<AssetServer>) {
    spawn_wave(0, commands, asset_server);
}

fn spawn_wave(wave_id: usize, mut commands: Commands, asset_server: Res<AssetServer>) {
    let waves = get_waves();
    let wave = waves.get(wave_id).unwrap();

    let x_spacing = 0.5;
    let z_spacing = 1.0;

    for enemy in wave.enemies.iter() {
        commands
            .spawn(Enemy {})
            .insert(Velocity::default())
            .insert(SpatialBundle {
                transform: Transform::from_translation(Vec3::new(
                    enemy.position[0] as f32 * x_spacing,
                    0.0,
                    enemy.position[1] as f32 * z_spacing,
                )),
                ..Default::default()
            })
            .insert(Damageable {
                health: enemy.health,
                is_player: false,
            })
            .insert(RigidBody::Dynamic)
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

// TODO: Specify this in e.g. a JSON file later?
fn get_waves() -> Vec<Wave> {
    let waves: Vec<Wave> = vec![
        Wave {
            enemies: vec![
                EnemyInstance {
                    position: [-1, -1],
                    ship_type: EnemyType::Type1,
                    health: 2,
                },
                EnemyInstance {
                    position: [-1, -2],
                    ship_type: EnemyType::Type1,
                    health: 2,
                },
                EnemyInstance {
                    position: [1, -1],
                    ship_type: EnemyType::Type1,
                    health: 2,
                },
                EnemyInstance {
                    position: [1, -2],
                    ship_type: EnemyType::Type1,
                    health: 2,
                },
            ],
        },
        Wave {
            enemies: vec![
                EnemyInstance {
                    position: [-1, -1],
                    ship_type: EnemyType::Type1,
                    health: 2,
                },
                EnemyInstance {
                    position: [-1, -2],
                    ship_type: EnemyType::Type1,
                    health: 2,
                },
                EnemyInstance {
                    position: [1, -1],
                    ship_type: EnemyType::Type1,
                    health: 2,
                },
                EnemyInstance {
                    position: [1, -2],
                    ship_type: EnemyType::Type1,
                    health: 2,
                },
                EnemyInstance {
                    position: [2, -1],
                    ship_type: EnemyType::Type1,
                    health: 2,
                },
                EnemyInstance {
                    position: [2, -2],
                    ship_type: EnemyType::Type1,
                    health: 2,
                },
            ],
        },
    ];

    waves
}

fn update_enemies(
    mut ai_state: ResMut<EnemyAIState>,
    time: Res<Time>,
    mut enemies: Query<&mut Velocity, With<Enemy>>,
) {
    ai_state.move_timer -= time.delta_seconds();
    if ai_state.move_timer <= 0.0 {
        // Swap direction
        ai_state.moving_left = !ai_state.moving_left;
        ai_state.move_timer = ENEMY_MOVE_DURATION_S;
    }

    for mut enemy_vel in enemies.iter_mut() {
        enemy_vel.linvel.x = if ai_state.moving_left {
            -ENEMY_MOVE_VELOCITY
        } else {
            ENEMY_MOVE_VELOCITY
        };
    }
}

fn change_wave(
    commands: Commands,
    asset_server: Res<AssetServer>,
    mut ai_state: ResMut<EnemyAIState>,
    enemies: Query<With<Enemy>>,
) {
    if !enemies.is_empty() {
        return;
    }

    let waves = get_waves();
    ai_state.current_wave += 1;
    if ai_state.current_wave >= waves.len() as u32 {
        println!("Done with all waves!");
        return;
    }

    spawn_wave(ai_state.current_wave as usize, commands, asset_server);
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
