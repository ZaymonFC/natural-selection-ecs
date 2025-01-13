use bevy::prelude::*;

// TODO(Zaymon): I'm thinking that splitting up entities and systems for some things
// might be a bad idea.

const PLANT_CELL_SIZE: f32 = 5.0; // Size of each plant cell.
const SPAWN_AREA_SIZE: f32 = 500.0; // Entities will spawn within ±SPAWN_AREA_SIZE.

#[derive(Component)]
pub struct Predator;

#[derive(Component)]
pub struct Prey;

#[derive(Component)]
pub struct Plant;

#[derive(Component)]
pub struct Motion {
    pub direction: Vec2,
    pub speed: f32,
}

#[derive(Component)]
pub struct Energy {
    pub current: f32,
    pub max: f32,
    pub drain_rate: f32,
}

#[derive(Component)]
pub struct Traits {
    pub max_speed: f32,
    pub turn_speed: f32,
}

pub fn spawn_predator(commands: &mut Commands, position: Vec2, traits: Option<Traits>) -> Entity {
    commands
        .spawn((
            Sprite {
                color: Color::srgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(10.0, 10.0)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 0.0),
            Predator,
            Motion {
                direction: Vec2::new(1.0, 0.0),
                speed: 0.0,
            },
            traits.unwrap_or(Traits {
                max_speed: 100.0,
                turn_speed: 3.0,
            }),
        ))
        .id()
}

pub fn spawn_prey(commands: &mut Commands, position: Vec2, traits: Option<Traits>) -> Entity {
    commands
        .spawn((
            Sprite {
                color: Color::srgb(0.64, 0.16, 0.16),
                custom_size: Some(Vec2::new(7.5, 7.5)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 0.0),
            Prey,
            Motion {
                direction: Vec2::new(1.0, 0.0),
                speed: 0.0,
            },
            Traits {
                max_speed: 80.0,
                turn_speed: 10.0,
            },
            Energy {
                current: 100.0,
                max: 100.0,
                drain_rate: 25.0,
            },
        ))
        .id()
}

pub fn spawn_plant(commands: &mut Commands, position: Vec2) -> Entity {
    commands
        .spawn((
            Sprite {
                color: Color::srgb(0.0, 0.5, 0.0),
                custom_size: Some(Vec2::new(PLANT_CELL_SIZE, PLANT_CELL_SIZE)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 0.0),
            Plant,
        ))
        .id()
}

pub fn random_position() -> Vec2 {
    Vec2::new(
        rand::random::<f32>() * SPAWN_AREA_SIZE * 2.0 - SPAWN_AREA_SIZE,
        rand::random::<f32>() * SPAWN_AREA_SIZE * 2.0 - SPAWN_AREA_SIZE,
    )
}
