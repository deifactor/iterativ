mod tiles;

use crate::tiles::*;
use quicksilver::prelude::*;

struct Iterativ {
    tiles: Tiles,
}

impl State for Iterativ {
    fn new() -> Result<Iterativ> {
        let font = Font::from_bytes(include_bytes!("../static/white_rabbit.ttf").to_vec())?;
        let tiles = Tiles::render(&font)?;
        Ok(Iterativ { tiles })
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::WHITE)?;
        let player = &self.tiles.player;
        window.draw(&player.area().with_center((400, 300)), Img(player));
        Ok(())
    }
}

fn main() {
    run::<Iterativ>("Draw Geometry", Vector::new(800, 600), Settings::default());
}
