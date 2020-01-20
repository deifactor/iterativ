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
    /// Whether the position is blocked to pathfinding *because of entities*. This can be computed just by knowing the
    /// list of entities as well as some way to determine which entities block pathfinding, but
    /// doing it this way means the map itself can handle pathfinding logic.
    ///
    /// Note that this does not include things that are blocked due to tiles.
    pub blocked_by_entity: Vec<bool>,
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
            blocked_by_entity: vec![false; vec_size],
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
        self.blocked_by_entity = vec![false; self.size()];
        for entities in &mut self.entities {
            entities.clear();
        }
    }

    pub fn blocked(&self, x: i32, y: i32) -> bool {
        let idx = self.idx(x, y);
        self.tiles[idx].is_solid() || self.blocked_by_entity[idx]
    }
}
