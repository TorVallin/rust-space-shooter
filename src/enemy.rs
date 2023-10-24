use bevy::prelude::Component;

#[derive(Component)]
pub struct Enemy {
    pub shot_cooldown_timer: f32,
}
