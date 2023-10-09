use bevy::{
    prelude::{
        App, AssetServer, Camera3dBundle, Commands, Component, Entity, Input, KeyCode, Mesh,
        Plugin, PointLight, PointLightBundle, Quat, Query, Res, ResMut, Resource, Startup,
        Transform, Update, Vec3, With,
    },
    scene::SceneBundle,
    time::{Time, Timer, TimerMode},
    transform::TransformBundle,
    DefaultPlugins,
};
use bevy_rapier3d::{
    prelude::{
        ActiveEvents, Ccd, Collider, NoUserData, RapierContext, RapierPhysicsPlugin, RigidBody,
        Sensor, Sleeping, GravityScale,
    },
    render::RapierDebugRenderPlugin,
};

#[derive(Default)]
struct Player {
    entity: Option<Entity>,
    lives: u32,
}

#[derive(Resource, Default)]
struct GameState {
    player: Player,
    score: u32,
}

#[derive(Component)]
struct EnemyBullet {}

fn setup_cameras(mut commands: Commands, _: ResMut<GameState>) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6.0, 2.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..Default::default()
    });
}

fn setup_game_state(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game: ResMut<GameState>,
) {
    game.score = 0;
    game.player.lives = 3;

    game.player.entity = Some(
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

    commands
        .spawn(Collider::cuboid(0.5, 1.0, 2.0))
        .insert(RigidBody::Fixed)
        .insert(Sensor)
        .insert(EnemyBullet {})
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(TransformBundle::from(Transform::from_xyz(2.0, 0.0, 2.0)));
}

fn player_controls(
    _: Commands,
    input: Res<Input<KeyCode>>,
    game: ResMut<GameState>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
) {
    let player = game.player.entity.unwrap();

    let mut transform = transforms.get_mut(player).unwrap();
    let mut translation = transform.translation;

    let move_speed = 3.0;
    // Move left and right with A/D
    if input.pressed(KeyCode::A) {
        translation.x -= move_speed * time.delta_seconds();
        *transform = Transform {
            translation,
            rotation: transform.rotation,
            ..Default::default()
        }
    }
    if input.pressed(KeyCode::D) {
        translation.x += move_speed * time.delta_seconds();
        *transform = Transform {
            translation,
            rotation: transform.rotation,
            ..Default::default()
        }
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_obj::ObjPlugin))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .init_resource::<GameState>()
        .add_systems(Startup, (setup_cameras, setup_game_state))
        .add_systems(Update, player_controls)
        .add_systems(Update, check_intersections)
        .run();
}

fn check_intersections(
    game: ResMut<GameState>,
    rapier_context: Res<RapierContext>,
    enemy_bullets: Query<Entity, (With<Collider>, With<EnemyBullet>)>,
) {
    for bullet in &enemy_bullets {
        let player = game.player.entity.unwrap();

        // Checks for intersections between the player and the enemy bullet
        if rapier_context.intersection_pair(player, bullet) == Some(true) {
            println!(
                "The entities {:?} and {:?} have intersecting colliders!",
                player, bullet
            );
        }
    }
}
