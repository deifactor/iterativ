use quicksilver::prelude::*;
use std::collections::HashMap;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum TileId {
    Player,
}

pub struct Tiles {
    tiles: HashMap<TileId, Image>,
    pub tile_size: Vector,
}

impl Tiles {
    pub fn render(font: &Font) -> Result<Self> {
        let player = font.render("@", &FontStyle::new(14.0, Color::BLACK))?;
        let mut tiles = HashMap::new();
        let tile_size = player.area().size();
        tiles.insert(TileId::Player, player);
        Ok(Tiles { tiles, tile_size })
    }

    pub fn tile(&self, id: TileId) -> &Image {
        &self.tiles[&id]
    }
}
