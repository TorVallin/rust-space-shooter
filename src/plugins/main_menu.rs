use bevy::{
    prelude::{
        default, in_state, App, AssetServer, BuildChildren, Button, ButtonBundle, Changed, Color,
        Commands, Component, IntoSystemConfigs, NextState, NodeBundle, OnEnter, OnExit, Plugin,
        Query, Res, ResMut, Startup, TextBundle, Update, With, Entity, DespawnRecursiveExt,
    },
    text::TextStyle,
    ui::{AlignItems, BackgroundColor, BorderColor, Interaction, JustifyContent, Style, Val},
};

use crate::state::GameState;

const BUTTON_COLOR: Color = Color::rgb(0.15, 0.15, 0.15);
const BUTTON_PRESSED_COLOR: Color = Color::rgb(0.4, 0.7, 0.4);

#[derive(Component)]
pub struct MainUiRoot {}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), init_ui);
        app.add_systems(OnExit(GameState::Menu), destroy_ui);
        app.add_systems(Update, (update_buttons).run_if(in_state(GameState::Menu)));
    }
}

fn init_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    background_color: BUTTON_COLOR.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Play",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 45.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                });
        })
        .insert(MainUiRoot {});
}

fn destroy_ui(mut commands: Commands, mut root_query: Query<Entity, With<MainUiRoot>>) {
    for ui in root_query.iter() {
        commands.entity(ui).despawn_recursive();
    }
}

fn update_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BUTTON_PRESSED_COLOR.into();
                next_state.set(GameState::Game);
            }
            _ => {
                *color = BUTTON_COLOR.into();
            }
        }
    }
}
