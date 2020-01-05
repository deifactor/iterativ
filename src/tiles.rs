use quicksilver::prelude::*;

pub struct Tiles {
    pub player: Image,
    pub tile_size: Vector
}

impl Tiles {
    pub fn render(font: &Font) -> Result<Self> {
        let player = font.render("@", &FontStyle::new(14.0, Color::BLACK))?;
        let tile_size = player.area().size();
        Ok(Tiles { player, tile_size })
    }
}
