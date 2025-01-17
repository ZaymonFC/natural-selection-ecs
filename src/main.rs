use bevy::prelude::*;
use rand::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_simulation, setup_camera))
        // Stage configuration
        .configure_sets(
            Update,
            (
                SimulationSet::Decisions,
                SimulationSet::Actions,
                SimulationSet::Resolution,
            )
                .chain(),
        )
        .add_systems(Update, plant_growth)
        .run();
}

// Define our simulation stages
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum SimulationSet {
    Decisions,
    Actions,
    Resolution,
}

#[derive(Component, Debug, Reflect)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component, Debug, Reflect)]
struct Energy {
    value: f32,
}

// --- SETUP ------------------------------------------------------------------
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_simulation(mut commands: Commands) {
    create_plant(&mut commands, 0, 0);
}

// --- PLANTS -----------------------------------------------------------------
#[derive(Component, Debug, Reflect)]
struct Plant;

const GROWTH_CHANCE_PER_SECOND: f32 = 2.0; // Increased chance
const CARDINAL_WEIGHT: f32 = 0.8; // 80% chance for cardinal directions
const PLANT_SIZE: f32 = 5.0;
const PLANT_GAP: f32 = 6.0;

fn create_plant(commands: &mut Commands, x: i32, y: i32) {
    commands.spawn((
        Plant,
        Energy { value: 10.0 },
        Position { x, y },
        Sprite {
            color: Color::srgb(0.0, 1.0, 0.0),
            custom_size: Some(Vec2::new(PLANT_SIZE, PLANT_SIZE)),
            ..default()
        },
        Transform::from_xyz(x as f32 * PLANT_GAP, y as f32 * PLANT_GAP, 0.0),
    ));
}

// TODO(later): Plants should preference growing in straight lines (cardinal directions) based on neighboring plants.
fn plant_growth(plants: Query<&Position, With<Plant>>, mut commands: Commands, time: Res<Time>) {
    let mut rng = rand::thread_rng();
    let growth_chance = GROWTH_CHANCE_PER_SECOND * time.delta_secs();

    // Collect current plant positions
    let plant_positions: std::collections::HashSet<(i32, i32)> =
        plants.iter().map(|pos| (pos.x, pos.y)).collect();

    for plant_pos in plants.iter() {
        // Chance to grow based on framerate
        if rng.gen::<f32>() < growth_chance {
            // Pick random direction with bias toward cardinal
            let (dx, dy) = if rng.gen::<f32>() < CARDINAL_WEIGHT {
                // Cardinal directions
                match rng.gen_range(0..4) {
                    0 => (1, 0),
                    1 => (-1, 0),
                    2 => (0, 1),
                    _ => (0, -1),
                }
            } else {
                // Diagonal directions
                match rng.gen_range(0..4) {
                    0 => (1, 1),
                    1 => (-1, 1),
                    2 => (1, -1),
                    _ => (-1, -1),
                }
            };

            let x = plant_pos.x + dx;
            let y = plant_pos.y + dy;

            // Skip if position is already occupied
            if plant_positions.contains(&(x, y)) {
                continue;
            }

            commands.spawn((
                Plant,
                Energy { value: 10.0 },
                Position { x, y },
                Sprite {
                    color: Color::srgb(0.0, 1.0, 0.0),
                    custom_size: Some(Vec2::new(PLANT_SIZE, PLANT_SIZE)),
                    ..default()
                },
                Transform::from_xyz(x as f32 * PLANT_GAP, y as f32 * PLANT_GAP, 0.0),
            ));
        }
    }
}
