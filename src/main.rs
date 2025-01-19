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
                SimulationSet::Perception,
                SimulationSet::Decisions,
                SimulationSet::Actions,
            )
                .chain(),
        )
        // Plant systems
        .add_systems(Update, plant_growth.in_set(SimulationSet::Actions))
        .add_event::<PlantSpawnEvent>()
        .add_systems(Update, handle_plant_spawn)
        // Prey systems
        .add_event::<PreySpawnEvent>()
        .add_event::<IntentEvent>()
        .add_systems(Update, handle_prey_spawn)
        .add_systems(
            Update,
            (
                vision_system.in_set(SimulationSet::Perception),
                animal_behaviour_system.in_set(SimulationSet::Decisions),
                handle_intent_system.in_set(SimulationSet::Decisions),
                movement_system.in_set(SimulationSet::Actions),
            ),
        )
        // Add to main systems
        .add_event::<CollisionEvent>()
        .add_systems(
            Update,
            (
                check_collisions.in_set(SimulationSet::Perception),
                handle_collisions.in_set(SimulationSet::Actions),
            ),
        )
        .run();
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum SimulationSet {
    Perception,
    Decisions,
    Actions,
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

#[derive(Component, Debug, Reflect, Eq, PartialEq, Copy, Clone)]
enum EntityKind {
    Plant,
    Prey,
    Predator,
}

#[derive(Component, Debug, Reflect)]
struct Edible {
    nutrition: f32,
}

#[derive(Component, Debug, Reflect)]
struct Diet {
    eats: Vec<EntityKind>,
}

// --- SETUP ------------------------------------------------------------------
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_simulation(
    mut plant_events: EventWriter<PlantSpawnEvent>,
    mut prey_events: EventWriter<PreySpawnEvent>,
    window_query: Query<&Window>,
) {
    let random_screen_positions = {
        || {
            let window = window_query.single();
            let mut rng = rand::thread_rng();
            let width_units = ((window.width() / PLANT_GAP) / 2.0) as i32;
            let height_units = ((window.height() / PLANT_GAP) / 2.0) as i32;

            std::iter::repeat_with(move || {
                let x = rng.gen_range(-width_units..width_units);
                let y = rng.gen_range(-height_units..height_units);
                (x, y)
            })
        }
    };

    random_screen_positions().take(10).for_each(|(x, y)| {
        plant_events.send(PlantSpawnEvent { x, y });
    });

    random_screen_positions().take(100).for_each(|(x, y)| {
        prey_events.send(PreySpawnEvent { x, y });
    });
}

// --- Animals ----------------------------------------------------------------
#[derive(Component, Debug, Reflect)]
struct Motion {
    direction: Vec2,
    speed: f32,
}

#[derive(Event)]
enum IntentEvent {
    Eat(Entity, Entity),    // (prey_entity, target_entity)
    Explore(Entity),        // prey_entity
    Escape(Entity, Entity), // (prey_entity, threat_entity)
}

#[derive(Component, Debug, Reflect)]
enum MotionBehavior {
    Seek { target: Entity },
    Evade { target: Entity },
    Wander,
}

#[derive(Component, Debug, Reflect)]
struct Behaves;

#[derive(Event)]
struct PreySpawnEvent {
    x: i32,
    y: i32,
}

fn create_prey(commands: &mut Commands, x: i32, y: i32) {
    commands.spawn((
        EntityKind::Prey,
        Behaves,
        Energy { value: 20.0 },
        Diet {
            eats: vec![EntityKind::Plant],
        },
        Edible { nutrition: 20.0 },
        Position { x, y },
        Motion {
            direction: Vec2::ZERO,
            speed: 50.0,
        },
        Vision {
            range: 10.0,
            visible_entities: Vec::new(),
        },
        MotionBehavior::Wander,
        Sprite {
            color: Color::srgb(0.0, 0.0, 1.0),
            custom_size: Some(Vec2::new(PLANT_SIZE * 1.2, PLANT_SIZE * 1.2)),
            ..default()
        },
        Transform::from_xyz(x as f32 * PLANT_GAP, y as f32 * PLANT_GAP, 0.0),
    ));
}

fn handle_prey_spawn(mut commands: Commands, mut event_reader: EventReader<PreySpawnEvent>) {
    for event in event_reader.read() {
        create_prey(&mut commands, event.x, event.y);
    }
}

// --- VISION -----------------------------------------------------------------
#[derive(Component, Debug)]
struct Vision {
    range: f32,                                       // Vision radius in grid units.
    visible_entities: Vec<(Entity, EntityKind, f32)>, // Entity, type, distance
}

fn vision_system(
    mut creatures: Query<(&Position, &mut Vision, &Diet)>,
    entities: Query<(Entity, &Position, &EntityKind, Option<&Edible>)>,
) {
    for (pos, mut vision, diet) in creatures.iter_mut() {
        vision.visible_entities.clear();

        for (entity, other_pos, kind, edible) in entities.iter() {
            let dx = pos.x - other_pos.x;
            let dy = pos.y - other_pos.y;
            let dist_sq = dx * dx + dy * dy;
            let range_sq = (vision.range * vision.range) as i32;

            if dist_sq < range_sq {
                if diet.eats.contains(kind) && edible.is_some() {
                    vision
                        .visible_entities
                        .push((entity, *kind, (dist_sq as f32).sqrt()));
                }
            }
        }
    }
}

fn animal_behaviour_system(
    query: Query<(Entity, &Vision, &Diet), With<Behaves>>,
    mut intent_events: EventWriter<IntentEvent>,
) {
    for (entity, vision, diet) in query.iter() {
        if !vision.visible_entities.is_empty() {
            // Find closest food entity
            let closest_food = vision
                .visible_entities
                .iter()
                .filter(|(_, kind, _)| diet.eats.contains(kind))
                .next();

            if let Some((food_entity, _, _)) = closest_food {
                intent_events.send(IntentEvent::Eat(entity, *food_entity));
                continue;
            }
        }
        intent_events.send(IntentEvent::Explore(entity));
    }
}

fn handle_intent_system(
    mut intents: EventReader<IntentEvent>,
    mut entities: Query<&mut MotionBehavior>,
) {
    for intent in intents.read() {
        match intent {
            IntentEvent::Eat(entity, target) => {
                if let Ok(mut behavior) = entities.get_mut(*entity) {
                    *behavior = MotionBehavior::Seek { target: *target };
                }
            }
            IntentEvent::Explore(entity) => {
                if let Ok(mut behavior) = entities.get_mut(*entity) {
                    *behavior = MotionBehavior::Wander;
                }
            }
            IntentEvent::Escape(entity, threat) => {
                if let Ok(mut behavior) = entities.get_mut(*entity) {
                    *behavior = MotionBehavior::Evade { target: *threat };
                }
            }
        }
    }
}

// --- PHYSICAL SYSTEMS -------------------------------------------------------
// - Add energy drain.

#[derive(Event)]
struct CollisionEvent(Entity, Entity); // (entity_a, entity_b)

// Simple collision detection
fn check_collisions(
    positions: Query<(Entity, &Position, &EntityKind)>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let entities: Vec<_> = positions.iter().collect();

    for (i, (entity_a, pos_a, _)) in entities.iter().enumerate() {
        for (entity_b, pos_b, _) in entities[i + 1..].iter() {
            if pos_a.x == pos_b.x && pos_a.y == pos_b.y {
                collision_events.send(CollisionEvent(*entity_a, *entity_b));
            }
        }
    }
}

fn handle_collisions(
    mut collisions: EventReader<CollisionEvent>,
    mut commands: Commands,
    mut query: Query<(&EntityKind, &Diet, &Edible, &mut Energy)>,
) {
    for CollisionEvent(entity_a, entity_b) in collisions.read() {
        let (nutrition_a, nutrition_b, can_a_eat_b, can_b_eat_a) =
            match (query.get(*entity_a), query.get(*entity_b)) {
                (Ok(a), Ok(b)) => (
                    a.2.nutrition,
                    b.2.nutrition,
                    a.1.eats.contains(b.0),
                    b.1.eats.contains(a.0),
                ),
                _ => continue,
            };

        if can_a_eat_b {
            eat(&mut commands, &mut query, *entity_a, *entity_b, nutrition_b);
        } else if can_b_eat_a {
            eat(&mut commands, &mut query, *entity_b, *entity_a, nutrition_a);
        }
    }
}

fn eat(
    commands: &mut Commands,
    query: &mut Query<(&EntityKind, &Diet, &Edible, &mut Energy)>,
    eater: Entity,
    eaten: Entity,
    nutrition: f32,
) {
    if let Ok(mut eater_energy) = query.get_mut(eater) {
        println!(
            "Entity {:?} ate entity {:?} for {} energy",
            eater, eaten, nutrition
        );
        eater_energy.3.value += nutrition;
        commands.entity(eaten).despawn();
    }
}

fn movement_system(
    mut param_set: ParamSet<(
        Query<(&MotionBehavior, &mut Motion, &mut Transform, &mut Position)>,
        Query<(Entity, &Position)>,
    )>,
    time: Res<Time>,
) {
    let mut rng = rand::thread_rng();

    let positions: Vec<(Entity, Position)> = param_set
        .p1()
        .iter()
        .map(|(e, p)| (e, Position { x: p.x, y: p.y }))
        .collect();

    for (behavior, mut motion, mut transform, mut position) in param_set.p0().iter_mut() {
        match behavior {
            MotionBehavior::Wander => {
                if rng.gen::<f32>() < 0.1 {
                    let angle = rng.gen::<f32>() * std::f32::consts::TAU; // Full 360° rotation
                    let change = Vec2::new(angle.cos(), angle.sin());
                    motion.direction = (motion.direction + change * 0.1).normalize();
                }
            }
            MotionBehavior::Seek { target } => {
                if let Some((_, target_pos)) = positions.iter().find(|(e, _)| e == target) {
                    let dx = target_pos.x - position.x;
                    let dy = target_pos.y - position.y;
                    let dir = Vec2::new(dx as f32, dy as f32);
                    if dir != Vec2::ZERO {
                        motion.direction = dir.normalize();
                    }
                }
            }
            MotionBehavior::Evade { target } => {
                if let Some((_, target_pos)) = positions.iter().find(|(e, _)| e == target) {
                    let dx = position.x - target_pos.x;
                    let dy = position.y - target_pos.y;
                    motion.direction = Vec2::new(dx as f32, dy as f32).normalize();
                }
            }
        }

        let movement = motion.direction * motion.speed * time.delta_secs();
        transform.translation.x += movement.x;
        transform.translation.y += movement.y;

        position.x = (transform.translation.x / PLANT_GAP).round() as i32;
        position.y = (transform.translation.y / PLANT_GAP).round() as i32;
    }
}

// --- PLANTS -----------------------------------------------------------------
#[derive(Component, Debug, Reflect)]
struct Plant;

#[derive(Event)]
struct PlantSpawnEvent {
    x: i32,
    y: i32,
}

// Growth parameters.
const GROWTH_CHANCE_PER_SECOND: f32 = 1.4; // Increased chance
const CARDINAL_WEIGHT: f32 = 0.9; // 80% chance for cardinal directions

// Sizing.
const PLANT_SIZE: f32 = 5.0;
const PLANT_GAP: f32 = 6.0;

// -- Creation.
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
        EntityKind::Plant,
        Diet { eats: vec![] },
        Edible { nutrition: 5.0 },
        Transform::from_xyz(x as f32 * PLANT_GAP, y as f32 * PLANT_GAP, 0.0),
    ));
}

fn handle_plant_spawn(mut commands: Commands, mut event_reader: EventReader<PlantSpawnEvent>) {
    for event in event_reader.read() {
        create_plant(&mut commands, event.x, event.y);
    }
}

// -- Systems.
fn plant_growth(
    plants: Query<&Position, With<Plant>>,
    mut events: EventWriter<PlantSpawnEvent>,
    time: Res<Time>,
) {
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
                // Cardinal directions with line growth bias
                let cardinal_dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
                let weighted_dirs: Vec<((i32, i32), f32)> = cardinal_dirs
                    .iter()
                    .map(|&(dx, dy)| {
                        let weight =
                            if plant_positions.contains(&(plant_pos.x - dx, plant_pos.y - dy)) {
                                16.0
                            } else {
                                1.0
                            };
                        ((dx, dy), weight)
                    })
                    .collect();

                let total_weight: f32 = weighted_dirs.iter().map(|(_, w)| w).sum();
                let mut choice = rng.gen::<f32>() * total_weight;
                let mut selected = cardinal_dirs[0];

                for ((dx, dy), weight) in weighted_dirs {
                    choice -= weight;
                    if choice <= 0.0 {
                        selected = (dx, dy);
                        break;
                    }
                }
                selected
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

            events.send(PlantSpawnEvent { x, y });
        }
    }
}
