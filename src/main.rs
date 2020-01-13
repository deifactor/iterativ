mod geometry;
mod tiles;

use crate::geometry::*;
use crate::tiles::*;
use quicksilver::prelude::*;
use specs::{prelude::*, Component};
use std::panic;
use stdweb::console;

#[derive(Component, Debug, Copy, Clone)]
struct PlayerId(pub Entity);

#[derive(Component, Debug, Copy, Clone)]
struct Position(pub Point<i32>);

pub enum Action {
    Move { dx: i32, dy: i32 },
}

#[derive(Component, Debug, Clone)]
/// If something should be drawn in the world, what's its tile? This is *not* the underlying Image,
/// since that's wrapped in an Rc and so it's not thread-safe.
struct Visible {
    pub tile_id: TileId,
}

pub struct Engine {
    pub world: World,
    player_action: Option<Action>,
}

impl Engine {
    pub fn new() -> Self {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Visible>();
        Engine {
            world,
            player_action: None,
        }
    }

    pub fn set_action(&mut self, action: Action) {
        self.player_action = Some(action);
    }

    pub fn perform(&mut self, entity: Entity, action: Action) {
        match action {
            Action::Move { dx, dy } => {
                let mut pos_storage = self.world.write_storage::<Position>();
                let pos = pos_storage
                    .get_mut(entity)
                    .expect("can't move something without a position");
                pos.0.x += dx;
                pos.0.y += dy;
            }
        }
    }

    pub fn player_act(&mut self) -> bool {
        let player: PlayerId = *self.world.fetch();
        if let Some(action) = self.player_action.take() {
            self.perform(player.0, action);
            true
        } else {
            false
        }
    }
}

struct Iterativ {
    tiles: Tiles,
    state: Engine,
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
        let mut state = Engine::new();
        let player = state
            .world
            .create_entity()
            .with(Position(Point { x: 0, y: 0 }))
            .with(Visible {
                tile_id: TileId::Player,
            })
            .build();
        state.world.insert(PlayerId(player));
        Ok(Iterativ { tiles, state })
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;

        let positions = self.state.world.read_storage::<Position>();
        let visibles = self.state.world.read_storage::<Visible>();

        for (pos, vis) in (&positions, &visibles).join() {
            window.draw(&self.tile_rect(pos), Img(self.tiles.tile(vis.tile_id)));
        }
        Ok(())
    }

    fn event(&mut self, event: &Event, _window: &mut Window) -> Result<()> {
        if let Event::Key(key, ButtonState::Pressed) = event {
            match key {
                Key::H => self.state.set_action(Action::Move { dx: -1, dy: 0 }),
                Key::J => self.state.set_action(Action::Move { dx: 0, dy: 1 }),
                Key::K => self.state.set_action(Action::Move { dx: 0, dy: -1 }),
                Key::L => self.state.set_action(Action::Move { dx: 1, dy: 0 }),
                _ => (),
            }
        }
        Ok(())
    }

    fn update(&mut self, _window: &mut Window) -> Result<()> {
        self.state.player_act();
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
