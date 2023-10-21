use bevy::{
    prelude::{
        AssetServer, BuildChildren, Commands, Plugin, Res, SpatialBundle, Startup, Transform, Vec3,
    },
    scene::SceneBundle,
};
use bevy_rapier3d::prelude::{ActiveEvents, Collider, GravityScale, RigidBody};

use crate::{combat::Damageable, enemy::Enemy};

pub struct EnemyWavePlugin;

pub struct Wave {
    enemies: Vec<EnemyInstance>,
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
        app.add_systems(Startup, init_enemy_waves);
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
    let waves: Vec<Wave> = vec![Wave {
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
    }];

    waves
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
