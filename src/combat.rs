use bevy::prelude::Component;

#[derive(Component)]
pub struct Health {
    pub health: u32,
}

#[derive(Component)]
pub struct EnemyBullet {}

#[derive(Component)]
pub struct PlayerBullet {}

#[derive(Component)]
pub struct Bullet {
    pub up_direction: bool,
    pub velocity: f32,
    pub damage: u32,
}
