mod geometry;
mod tiles;

use crate::geometry::*;
use crate::tiles::*;
use quicksilver::prelude::*;
use std::panic;
use stdweb::console;

#[derive(Debug, Copy, Clone)]
struct Position(pub Point<i32>);

#[derive(Debug, Clone)]
/// If something should be drawn in the world, what's its tile? We can just store the image
/// directly, since it's just an Rc so it's cheap to maintain.
struct Tile(pub Image);

pub struct World {
    pub ecs: recs::Ecs,
    pub player: recs::EntityId,
}

impl World {
    pub fn new() -> Self {
        let mut ecs = recs::Ecs::new();
        let player = ecs.create_entity();
        World { ecs, player }
    }
}

struct Iterativ {
    tiles: Tiles,
    world: World,
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
        let mut world = World::new();
        world
            .ecs
            .set(world.player, Position(Point { x: 0, y: 0 }))
            .expect("missing player");
        world
            .ecs
            .set(world.player, Tile(tiles.player.clone()))
            .expect("missing player");
        Ok(Iterativ { tiles, world })
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::WHITE)?;
        let player = &self.tiles.player;
        let pos = self
            .world
            .ecs
            .borrow::<Position>(self.world.player)
            .expect("no position?");
        let image = self
            .world
            .ecs
            .borrow::<Tile>(self.world.player)
            .expect("no image?");
        window.draw(&self.tile_rect(pos), Img(&image.0));
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
