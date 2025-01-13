use bevy::prelude::*;
use rand::random;

use crate::entities::{Motion, Plant, Prey, Traits};

const PREY_VISION_RANGE: f32 = 100.0;
const RANDOM_TURN_RATE: f32 = 1.0;

pub fn prey_movement(
    mut query_set: ParamSet<(
        Query<(&mut Transform, &mut Motion, &Traits), With<Prey>>,
        Query<&Transform, With<Plant>>,
    )>,
    time: Res<Time>,
) {
    let plants: Vec<Vec3> = query_set
        .p1()
        .iter()
        .map(|transform| transform.translation)
        .collect();

    for (mut transform, mut motion, traits) in query_set.p0().iter_mut() {
        let target_direction = if let Some((direction, _)) = plants
            .iter()
            .map(|plant_pos| {
                let direction = *plant_pos - transform.translation;
                (direction, direction.length())
            })
            .filter(|(_, distance)| *distance <= PREY_VISION_RANGE)
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        {
            direction.truncate().normalize()
        } else {
            let random_turn = (random::<f32>() - 0.5) * RANDOM_TURN_RATE;
            let current_angle = motion.direction.y.atan2(motion.direction.x);
            let new_angle = current_angle + random_turn;
            Vec2::new(new_angle.cos(), new_angle.sin())
        };

        let current_angle = motion.direction.y.atan2(motion.direction.x);
        let target_angle = target_direction.y.atan2(target_direction.x);
        let angle_diff = (target_angle - current_angle + std::f32::consts::PI)
            % (2.0 * std::f32::consts::PI)
            - std::f32::consts::PI;

        let rotation = angle_diff.signum() * traits.turn_speed * time.delta_secs();
        let new_angle = current_angle
            + rotation.clamp(
                -traits.turn_speed * time.delta_secs(),
                traits.turn_speed * time.delta_secs(),
            );

        motion.direction = Vec2::new(new_angle.cos(), new_angle.sin());
        motion.speed = traits.max_speed;

        transform.translation += (motion.direction * motion.speed * time.delta_secs()).extend(0.0);
    }
}
