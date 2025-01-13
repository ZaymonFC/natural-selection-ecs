use crate::entities::{spawn_plant, Plant};
use bevy::prelude::*;

pub const PLANT_GROWTH_RATE: f32 = 0.8; // Growth chance per second.
pub const PLANT_CELL_SIZE: f32 = 5.0; // Size of each plant cell.

pub fn plant_growth(
    mut commands: Commands,
    query: Query<&Transform, With<Plant>>,
    time: Res<Time>,
) {
    for plant_transform in query.iter() {
        if rand::random::<f32>() < PLANT_GROWTH_RATE * time.delta_secs() {
            let angle = (rand::random::<f32>() * 4.0).floor() * std::f32::consts::FRAC_PI_2;
            let offset = Vec2::new(angle.cos(), angle.sin()) * PLANT_CELL_SIZE;
            let new_pos = plant_transform.translation.truncate() + offset;

            let is_position_taken = query.iter().any(|transform| {
                transform.translation.truncate().distance(new_pos) < PLANT_CELL_SIZE / 2.0
            });

            if !is_position_taken {
                spawn_plant(&mut commands, new_pos);
            }
        }
    }
}
