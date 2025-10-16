use crate::start;
use avian3d::prelude::*;
use bevy::prelude::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    fn deviceMotionEvent() -> web_sys::DeviceMotionEvent;
}

#[cfg(feature = "web")]
#[wasm_bindgen]
pub fn start_web(width: u32, height: u32) {
    start(width, height);
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, handle_motion);
}

fn handle_motion(mut gravity: ResMut<Gravity>, mut camera_forces: Query<Forces, With<Camera3d>>) {
    let event = deviceMotionEvent();
    if let Some(acceleration) = event.acceleration() {
        if let Some(acceleration_including_gravity) = event.acceleration_including_gravity() {
            gravity.0 = Vec3::new(
                (acceleration_including_gravity.x().unwrap_or(0.) - acceleration.x().unwrap_or(0.))
                    as f32,
                (acceleration_including_gravity.y().unwrap_or(0.) - acceleration.y().unwrap_or(0.))
                    as f32,
                (acceleration_including_gravity.z().unwrap_or(0.) - acceleration.z().unwrap_or(0.))
                    as f32,
            );
        }
        let acceleration = Vec3::new(
            acceleration.x().unwrap_or(0.) as f32,
            acceleration.y().unwrap_or(0.) as f32,
            acceleration.z().unwrap_or(0.) as f32,
        );
        for mut forces in &mut camera_forces {
            forces.apply_force(acceleration * 5.);
        }
    }
}
