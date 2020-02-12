mod ai;
mod engine;
mod event_log;
mod geometry;
mod map;
mod tiles;

use crate::ai::{AIComponent, PlayerAI, Swarm};
use crate::engine::*;
use crate::event_log::EventLogRenderer;
use crate::geometry::*;
use crate::tiles::*;
use quicksilver::prelude::*;
use specs::prelude::*;

const WIDTH: i32 = 80;
const HEIGHT: i32 = 40;
const MAP_HEIGHT: i32 = 30;
const TILE_SIZE: i32 = 14;

struct Iterativ {
    tiles: Tiles,
    state: Engine,
    log_renderer: EventLogRenderer,
}

impl Iterativ {
    fn tile_rect(&self, pos: &Position) -> Rectangle {
        Rectangle::new(
            self.tiles.tile_size.times((pos.0.x, pos.0.y)),
            self.tiles.font_size,
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
            .with(Name {
                name: "you".to_string(),
            })
            .with(Position(Point { x: 5, y: 5 }))
            .with(AIComponent(Box::new(PlayerAI)))
            .with(Visible {
                tile_id: TileId::Player,
            })
            .with(Initiative::new(10))
            .with(BlocksMovement)
            .build();
        state
            .world
            .create_entity()
            .with(Name {
                name: "the swarmer".to_string(),
            })
            .with(Position(Point { x: 0, y: 0 }))
            .with(Visible {
                tile_id: TileId::Grunt,
            })
            .with(Initiative::new(20))
            .with(BlocksMovement)
            .with(AIComponent(Box::new(Swarm { target: player })))
            .build();
        state.world.insert(PlayerId(player));
        state.world.insert(event_log::EventLog::new());
        state.world.insert(map::Map::new(WIDTH, HEIGHT));

        let log_renderer = EventLogRenderer::new(
            Rectangle::new(
                (0, MAP_HEIGHT * TILE_SIZE),
                (WIDTH * TILE_SIZE, (HEIGHT - MAP_HEIGHT) * TILE_SIZE),
            ),
            font,
            FontStyle::new(TILE_SIZE as f32, Color::WHITE),
        )?;
        Ok(Iterativ {
            tiles,
            state,
            log_renderer,
        })
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::BLACK)?;

        let positions = self.state.world.read_storage::<Position>();
        let visibles = self.state.world.read_storage::<Visible>();

        for (pos, vis) in (&positions, &visibles).join() {
            window.draw(&self.tile_rect(pos), Img(self.tiles.tile(vis.tile_id)));
        }

        let event_log = self.state.world.fetch::<event_log::EventLog>();
        let names = self.state.world.read_storage::<Name>();
        self.log_renderer.render(&event_log, &names, window)?;

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
        self.state.tick();
        self.state.world.maintain();
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
fn wasm_panic_hook(info: &std::panic::PanicInfo) {
    use stdweb::console;
    console!(error, info.to_string());
}

#[cfg(target_arch = "wasm32")]
fn setup_logging() {
    web_logger::init();
}

#[cfg(not(target_arch = "wasm32"))]
fn setup_logging() {
    simple_logger::init_with_level(log::Level::Trace).expect("couldn't init simple_logger");
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    std::panic::set_hook(Box::new(wasm_panic_hook));
    setup_logging();

    let size = Vector::new((WIDTH * TILE_SIZE) as i32, (HEIGHT * TILE_SIZE) as i32);
    // Setting min_size and max_size here tells i3 that this wants to be a floating window. the
    let settings = Settings {
        min_size: Some(size),
        max_size: Some(size),
        ..Settings::default()
    };
    run::<Iterativ>("Draw Geometry", size, settings);
}
