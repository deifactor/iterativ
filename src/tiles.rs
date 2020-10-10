use quicksilver::{
    geom::Vector,
    graphics::{Color, FontRenderer, Graphics},
    Result,
};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum TileId {
    Player,
    Grunt,
}

impl TileId {
    fn text(&self) -> &'static str {
        match self {
            TileId::Player => "@",
            TileId::Grunt => "g",
        }
    }

    fn color(&self) -> Color {
        Color::WHITE
    }
}

pub struct Tiles {
    pub renderer: FontRenderer,
}

impl Tiles {
    pub fn draw(&mut self, gfx: &mut Graphics, tile: TileId, position: Vector) -> Result<()> {
        self.renderer
            .draw(gfx, tile.text(), tile.color(), position)?;
        Ok(())
    }
}
