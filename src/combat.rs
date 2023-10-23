use bevy::prelude::{Component, Vec3};

#[derive(Component)]
pub struct EntityDeath {
    pub position: Vec3,  // Where the death occured
    pub is_player: bool, // If it was the player (true) or an enemy that died (false)
}

#[derive(Component)]
pub struct ParticleHitEffect {
    pub position: Vec3, // Where the hit occured
    pub is_large: bool,
}

#[derive(Component)]
pub struct SmallHitEffect {}

#[derive(Component)]
pub struct LargeHitEffect {}

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
