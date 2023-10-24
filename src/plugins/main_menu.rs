use bevy::prelude::{App, Plugin, Startup, Commands};

struct MainMenuPlugin{}

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_ui);
    }
}

fn init_ui(mut commands: Commands) {

}