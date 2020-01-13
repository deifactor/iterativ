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
    pub font_size: Vector,
}

impl Tiles {
    pub fn render(font: &Font) -> Result<Self> {
        let style = &FontStyle::new(crate::TILE_SIZE as f32, Color::WHITE);
        let player = font.render("@", style)?;
        let grunt = font.render("g", style)?;
        let mut tiles = HashMap::new();
        let tile_size = (crate::TILE_SIZE as i32, crate::TILE_SIZE as i32).into();
        let font_size = player.area().size();
        tiles.insert(TileId::Player, player);
        tiles.insert(TileId::Grunt, grunt);
        Ok(Tiles {
            tiles,
            tile_size,
            font_size,
        })
    }

    pub fn tile(&self, id: TileId) -> &Image {
        &self.tiles[&id]
    }
}
