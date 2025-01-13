use bevy::prelude::*;
use std::collections::HashSet;

use crate::entities::{Energy, Plant, Prey};

const EATING_DISTANCE: f32 = 1.5;
const ENERGY_GAIN_FROM_PLANT: f32 = 10.0;

pub fn eating(
    mut commands: Commands,
    mut prey_query: Query<(&Transform, &mut Energy), With<Prey>>,
    plant_query: Query<(Entity, &Transform), With<Plant>>,
) {
    let mut eaten_plants = HashSet::new();

    for (prey_transform, mut prey_energy) in prey_query.iter_mut() {
        for (plant_entity, plant_transform) in plant_query.iter() {
            if !eaten_plants.contains(&plant_entity) {
                let distance = prey_transform
                    .translation
                    .distance(plant_transform.translation);
                if distance <= EATING_DISTANCE {
                    eaten_plants.insert(plant_entity);
                    prey_energy.current =
                        (prey_energy.current + ENERGY_GAIN_FROM_PLANT).min(prey_energy.max);
                    commands.entity(plant_entity).despawn(); // Despawn plant immediately when eaten
                    info!("Plant {:?} was eaten", plant_entity);
                    break; // Move to next prey after eating one plant
                }
            }
        }
    }
}
