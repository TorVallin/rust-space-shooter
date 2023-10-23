use bevy::prelude::{Assets, Commands, Component, Name, ResMut, Vec2, Vec3, Vec4};
use bevy_hanabi::{
    Attribute, ColorOverLifetimeModifier, EffectAsset, ExprWriter, Gradient, LinearDragModifier,
    ParticleEffectBundle, ScalarType, SetAttributeModifier, SetPositionSphereModifier,
    SetVelocitySphereModifier, ShapeDimension, SizeOverLifetimeModifier, Spawner,
};

use crate::combat::{LargeHitEffect, SmallHitEffect};

#[derive(Component)]
pub struct SmallExplosion;

#[derive(Component)]
pub struct LargeExplosion;

pub fn create_effect(
    name: &str,
    particle_count: f32,
    is_large: bool,
    effects: &mut ResMut<Assets<EffectAsset>>,
    commands: &mut Commands,
) {
    let spawner = Spawner::once(particle_count.into(), false);

    let writer = ExprWriter::new();

    let age = writer.lit(0.).uniform(writer.lit(0.2)).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);
    let lifetime = writer.lit(0.25).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let drag = writer.lit(2.).expr();
    let update_drag = LinearDragModifier::new(drag);

    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient1.add_key(0.1, Vec4::new(4.0, 4.0, 0.0, 1.0));
    color_gradient1.add_key(0.9, Vec4::new(4.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(4.0, 0.0, 0.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(0.1));
    size_gradient1.add_key(0.3, Vec2::splat(0.1));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.5).expr(),
        dimension: ShapeDimension::Volume,
    };

    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: (writer.rand(ScalarType::Float) * writer.lit(5.0) + writer.lit(10.0)).expr(),
    };

    let effect = effects.add(
        EffectAsset::new(32768, spawner, writer.finish())
            .with_name(name)
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .update(update_drag)
            .render(ColorOverLifetimeModifier {
                gradient: color_gradient1,
            })
            .render(SizeOverLifetimeModifier {
                gradient: size_gradient1,
                screen_space_size: false,
            }),
    );

    if is_large {
        commands
            .spawn(ParticleEffectBundle::new(effect).with_spawner(spawner))
            .insert(Name::new(name.to_string()))
            .insert(LargeHitEffect {});
    } else {
        commands
            .spawn(ParticleEffectBundle::new(effect).with_spawner(spawner))
            .insert(Name::new(name.to_string()))
            .insert(SmallHitEffect {});
    }
}
