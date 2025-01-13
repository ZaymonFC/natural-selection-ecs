use crate::entities::Energy;
use bevy::prelude::*;

pub fn energy_drain(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Energy)>,
) {
    for (entity, mut energy) in query.iter_mut() {
        energy.current = (energy.current - energy.drain_rate * time.delta_secs()).max(0.0);
        if energy.current <= 0.0 {
            commands.entity(entity).despawn();
            info!("Entity {:?} died due to energy depletion", entity);
        }
    }
}
