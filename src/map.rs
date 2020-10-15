use specs::Entity;

use crate::geometry::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TileType {
    Floor,
    Wall,
}

impl TileType {
    /// Whether the tile should be considered solid for pathfinding purposes.
    pub fn is_solid(&self) -> bool {
        match self {
            TileType::Floor => false,
            TileType::Wall => true,
        }
    }
}

pub struct Map {
    width: i32,
    height: i32,
    // Implementation note: the vecs are stored in row-major order (so position 0 is top-left,
    // position 1 is to the right of that, etc).
    /// The tile that's located at each position of the map.
    tiles: Vec<TileType>,
    /// All entities located at a given position.
    pub entities: Vec<Vec<Entity>>,
    /// For each tile that has a blocking entity on it, this returns that entity.
    pub blockers: Vec<Option<Entity>>,
}

impl Map {
    pub fn new(width: i32, height: i32) -> Self {
        assert!(width >= 0, "negative width");
        assert!(height >= 0, "negative height");
        let vec_size = (width * height) as usize;
        Map {
            width,
            height,
            tiles: vec![TileType::Floor; vec_size],
            blockers: vec![None; vec_size],
            entities: vec![vec![]; vec_size],
        }
    }

    pub fn size(&self) -> usize {
        (self.width * self.height) as usize
    }

    fn idx(&self, point: WorldPoint) -> usize {
        assert!(point.x >= 0);
        assert!(point.y >= 0);
        (self.width * point.y + point.x) as usize
    }

    /// Clears out all entities, including the cached tile blocking information. Does not modify
    /// the tiles themselves.
    pub fn clear_entities(&mut self) {
        self.blockers = vec![None; self.size()];
        for entities in &mut self.entities {
            entities.clear();
        }
    }

    pub fn add_entity(&mut self, point: WorldPoint, entity: Entity, blocks: bool) {
        let idx = self.idx(point);
        if blocks {
            self.blockers[idx] = Some(entity)
        }
        self.entities[idx].push(entity);
    }

    /// Is movement onto this tile blocked?
    pub fn is_blocked(&self, point: WorldPoint) -> bool {
        let idx = self.idx(point);
        self.tiles[idx].is_solid() || self.blockers[idx].is_some()
    }

    /// If motion onto this tile is blocked by a specific entity, returns that entity. Note that
    /// this can return None even if is_blocked is true if the map itself blocks movement.
    pub fn blockers(&self, point: WorldPoint) -> Option<Entity> {
        self.blockers[self.idx(point)]
    }
}
