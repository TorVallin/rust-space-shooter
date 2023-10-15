use bevy::prelude::Component;

#[derive(Component)]
pub struct Damageable {
    pub health: u32,
    pub is_player: bool,
}

#[derive(Component)]
pub struct Bullet {
    pub is_player_bullet: bool,
    pub up_direction: bool,
    pub velocity: f32,
    pub damage: u32,
}
