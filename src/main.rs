use bevy::{
    prelude::{
        App, AssetServer, Camera3dBundle, Commands, Component, Entity, Mesh, Plugin, PointLight,
        PointLightBundle, Query, Res, ResMut, Resource, Startup, Transform, Update, Vec3, With,
    },
    scene::SceneBundle,
    time::{Time, Timer, TimerMode},
    DefaultPlugins,
};

#[derive(Default)]
struct Player {
    entity: Option<Entity>,
    lives: u32,
    position: Vec3,
}

#[derive(Resource, Default)]
struct GameState {
    player: Player,
    score: u32,
}

fn setup_cameras(mut commands: Commands, mut game: ResMut<GameState>) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6.0, 2.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..Default::default()
    });
}

fn setup_game_state(mut commands: Commands, asset_server: Res<AssetServer>, mut game: ResMut<GameState>) {
    game.score = 0;
    game.player.lives = 3;

    game.player.entity = Some(
        commands
            .spawn(SceneBundle {
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 0.0),
                    ..Default::default()
                },
                scene: asset_server.load("Spaceship4/model.obj"),
                ..Default::default()
            })
            .id(),
    );

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..Default::default()
        },
        ..Default::default()
    });
}

fn player_controls(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game: ResMut<GameState>,
) {
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, bevy_obj::ObjPlugin))
        .init_resource::<GameState>()
        .add_systems(Startup, (setup_cameras, setup_game_state))
        .add_systems(Update, player_controls)
        .run();
}
