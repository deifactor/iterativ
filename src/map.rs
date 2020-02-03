use specs::Entity;

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

///
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

    pub fn idx(&self, x: i32, y: i32) -> usize {
        (self.width * y + x) as usize
    }

    pub fn from_idx(&self, idx: usize) -> (i32, i32) {
        ((idx as i32) % self.width, (idx as i32) / self.width)
    }

    /// Clears out all entities, including the cached tile blocking information. Does not modify
    /// the tiles themselves.
    pub fn clear_entities(&mut self) {
        self.blockers = vec![None; self.size()];
        for entities in &mut self.entities {
            entities.clear();
        }
    }

    pub fn blocked(&self, x: i32, y: i32) -> bool {
        let idx = self.idx(x, y);
        self.tiles[idx].is_solid() || self.blockers[idx].is_some()
    }
}
