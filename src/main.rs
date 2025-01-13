mod entities;
mod systems;

use crate::entities::{random_position, spawn_plant, spawn_predator, spawn_prey};
use crate::systems::{
    eating::eating,
    energy_drain::energy_drain,
    movement::prey_movement,
    plant_growth::plant_growth,
    spacial_grid::{update_spatial_grid, SpatialGrid},
};
use bevy::prelude::*;

const PREDATOR_COUNT: usize = 10;
const PREY_COUNT: usize = 180;
const PLANT_COUNT: usize = 500;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<SpatialGrid>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                prey_movement,
                eating,
                plant_growth,
                update_spatial_grid,
                energy_drain,
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // for _ in 0..PREDATOR_COUNT {
    //     spawn_predator(&mut commands, random_position(), None);
    // }

    for _ in 0..PREY_COUNT {
        spawn_prey(&mut commands, random_position(), None);
    }

    for _ in 0..PLANT_COUNT {
        spawn_plant(&mut commands, random_position());
    }
}
