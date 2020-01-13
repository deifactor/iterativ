use quicksilver::prelude::*;
use std::collections::HashMap;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum TileId {
    Player,
    Grunt,
}

pub struct Tiles {
    tiles: HashMap<TileId, Image>,
    pub tile_size: Vector,
}

impl Tiles {
    pub fn render(font: &Font) -> Result<Self> {
        let style = &FontStyle::new(14.0, Color::WHITE);
        let player = font.render("@", style)?;
        let grunt = font.render("g", style)?;
        let mut tiles = HashMap::new();
        let tile_size = player.area().size();
        tiles.insert(TileId::Player, player);
        tiles.insert(TileId::Grunt, grunt);
        Ok(Tiles { tiles, tile_size })
    }

    pub fn tile(&self, id: TileId) -> &Image {
        &self.tiles[&id]
    }
}
