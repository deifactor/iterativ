mod geometry;
mod tiles;

use crate::geometry::*;
use crate::tiles::*;
use quicksilver::prelude::*;
use specs::{prelude::*, Component};
use std::panic;
use stdweb::console;

#[derive(Component, Debug, Copy, Clone)]
struct Position(pub Point<i32>);

#[derive(Component, Debug, Clone)]
/// If something should be drawn in the world, what's its tile? This is *not* the underlying Image,
/// since that's wrapped in an Rc and so it's not thread-safe.
struct Visible {
    pub tile_id: TileId,
}

pub struct GameState {
    pub world: World,
}

impl GameState {
    pub fn new() -> Self {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Visible>();
        GameState { world }
    }
}

struct Iterativ {
    tiles: Tiles,
    state: GameState,
}

impl Iterativ {
    fn tile_rect(&self, pos: &Position) -> Rectangle {
        Rectangle::new(
            self.tiles.tile_size.times((pos.0.x, pos.0.y)),
            self.tiles.tile_size,
        )
    }
}

impl State for Iterativ {
    fn new() -> Result<Iterativ> {
        let font = Font::from_bytes(include_bytes!("../static/white_rabbit.ttf").to_vec())?;
        let tiles = Tiles::render(&font)?;
        let mut state = GameState::new();
        state
            .world
            .create_entity()
            .with(Position(Point { x: 0, y: 0 }))
            .with(Visible {
                tile_id: TileId::Player,
            })
            .build();
        Ok(Iterativ { tiles, state })
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::WHITE)?;

        let positions = self.state.world.read_storage::<Position>();
        let visibles = self.state.world.read_storage::<Visible>();

        for (pos, vis) in (&positions, &visibles).join() {
            window.draw(&self.tile_rect(pos), Img(self.tiles.tile(vis.tile_id)));
        }
        Ok(())
    }
}

fn panic_hook(info: &panic::PanicInfo) {
    console!(error, info.to_string());
}

fn main() {
    panic::set_hook(Box::new(panic_hook));
    run::<Iterativ>("Draw Geometry", Vector::new(800, 600), Settings::default());
}
