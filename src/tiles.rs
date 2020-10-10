use quicksilver::{
    geom::{Rectangle, Vector},
    graphics::{Color, Graphics, Image},
    Result,
};
use std::collections::HashMap;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum TileId {
    Player,
    Grunt,
}

pub struct Tiles {
    images: HashMap<TileId, Image>,
}

impl Tiles {
    pub async fn new(gfx: &Graphics) -> Result<Self> {
        let mut images = HashMap::new();
        images.insert(
            TileId::Player,
            Image::load(gfx, "sprites/player.png").await?,
        );
        images.insert(TileId::Grunt, Image::load(gfx, "sprites/grunt.png").await?);
        Ok(Self { images })
    }
    pub fn draw(&mut self, gfx: &mut Graphics, tile: TileId, position: Vector) -> Result<()> {
        let image = &self.images[&tile];
        gfx.draw_image(&image, Rectangle::new(position, image.size()));
        Ok(())
    }
}
