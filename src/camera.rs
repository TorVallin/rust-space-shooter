use bevy::{
    prelude::{Camera, Event, EventReader, Query, Res, ResMut, Resource, Transform, Vec3, With},
    time::Time,
};
use rand::Rng;

#[derive(Resource)]
pub struct CameraState {
    pub original_position: Vec3,
    pub shake_duration: f32,
    pub shake_intensity: f32,
}

#[derive(Event)]
pub struct CameraShakeEvent {
    pub intensity: f32,
}

pub fn on_hit_camera_shake(
    mut camera_state: ResMut<CameraState>,
    time: Res<Time>,
    mut er: EventReader<CameraShakeEvent>,
    mut camera: Query<&mut Transform, With<Camera>>,
) {
    let mut transform = camera.get_single_mut().unwrap();
    if camera_state.shake_duration > 0. {
        camera_state.shake_duration -= time.delta_seconds();
        if camera_state.shake_duration < 0. {
            transform.translation = camera_state.original_position;
            camera_state.shake_intensity = 0.;
        } else {
            let mut rng = rand::thread_rng();

            // Shake!
            let rand_x = rng.gen_range(-0.1..0.1) * camera_state.shake_intensity;
            let rand_z = rng.gen_range(-0.1..0.1) * camera_state.shake_intensity;
            transform.translation.x += rand_x;
            transform.translation.z += rand_z;
        }
    }

    for event in er.iter() {
        camera_state.shake_duration += 0.1;
        camera_state.shake_intensity += event.intensity;

        // Clamp intensity & duration to prevent drastic camera movement
        camera_state.shake_duration = f32::clamp(camera_state.shake_duration, 0.0, 0.25);
        camera_state.shake_intensity = f32::clamp(camera_state.shake_intensity, 0.0, 1.5);
    }
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            original_position: Vec3::new(0.0, 20.0, 2.0),
            shake_duration: 0.,
            shake_intensity: 0.,
        }
    }
}
