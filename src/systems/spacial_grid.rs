use bevy::{
    math::Vec2,
    prelude::{Component, Entity, Query, RemovedComponents, ResMut, Resource, Transform, With},
    utils::{HashMap, HashSet},
};

const GRID_CELL_SIZE: f32 = 10.0;

#[derive(Component)]
pub struct SpatialIndexed;

#[derive(Default, Resource)]
pub struct SpatialGrid {
    cells: HashMap<(i32, i32), HashSet<Entity>>,
}

fn to_grid_coords(pos: Vec2) -> (i32, i32) {
    (
        (pos.x / GRID_CELL_SIZE).floor() as i32,
        (pos.y / GRID_CELL_SIZE).floor() as i32,
    )
}

impl SpatialGrid {
    pub fn find_neighbors(&self, pos: Vec2, radius: i32) -> HashSet<Entity> {
        let grid_pos = to_grid_coords(pos);
        let mut neighbors = HashSet::new();

        for dx in -radius..=radius {
            for dy in -radius..=radius {
                if let Some(cell) = self.cells.get(&(grid_pos.0 + dx, grid_pos.1 + dy)) {
                    neighbors.extend(cell);
                }
            }
        }

        neighbors
    }

    pub fn find_nearest<'a, T: Component>(
        &self,
        pos: Vec2,
        search_radius: i32,
        query: &'a Query<(Entity, &Transform), With<T>>,
    ) -> Option<(Entity, f32)> {
        self.find_neighbors(pos, search_radius)
            .iter()
            .filter_map(|&entity| {
                query.get(entity).ok().map(|(e, transform)| {
                    let dist = pos.distance(transform.translation.truncate());
                    (e, dist)
                })
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }
}

pub fn update_spatial_grid(
    mut grid: ResMut<SpatialGrid>,
    query: Query<(Entity, &Transform), With<SpatialIndexed>>,
    mut removed: RemovedComponents<SpatialIndexed>,
) {
    // Clear previous grid state
    grid.cells.clear();

    // Remove entities that were despawned
    for entity in removed.read() {
        for cell in grid.cells.values_mut() {
            cell.remove(&entity);
        }
    }

    // Update grid with current positions
    for (entity, transform) in query.iter() {
        let pos = transform.translation.truncate();
        let grid_pos = to_grid_coords(pos);
        grid.cells.entry(grid_pos).or_default().insert(entity);
    }
}
