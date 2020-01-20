mod event_log;
mod geometry;
mod tiles;

use crate::event_log::EventLogRenderer;
use crate::geometry::*;
use crate::tiles::*;
use log::info;
use quicksilver::prelude::*;
use specs::{prelude::*, Component};

#[derive(Component, Debug, Copy, Clone)]
struct Initiative {
    pub current: i32,
    pub initial: i32,
}

const WIDTH: i32 = 80;
const HEIGHT: i32 = 40;
const MAP_HEIGHT: i32 = 30;
const TILE_SIZE: i32 = 14;

impl Initiative {
    pub fn new(initial: i32) -> Self {
        Initiative {
            current: initial,
            initial,
        }
    }

    /// Ticks down the initiative count. Returns true if the entity is ready (i.e., if its
    /// initiative reached 0). Note that this will also reset the initiative to its initial value.
    pub fn tick(&mut self) -> bool {
        self.current -= 1;
        if self.current <= 0 {
            self.current = self.initial;
            true
        } else {
            false
        }
    }
}

#[derive(Component, Default, Debug, Copy, Clone)]
#[storage(NullStorage)]
struct Ready;

trait AI: Send + Sync + std::fmt::Debug {
    fn decide(&mut self, world: &World, me: Entity) -> Action;
}

#[derive(Copy, Clone, Debug)]
struct Swarm {
    target: Entity,
}

impl AI for Swarm {
    fn decide(&mut self, world: &World, me: Entity) -> Action {
        let pos_component = world.read_component::<Position>();
        let to_target =
            pos_component.get(self.target).unwrap().0 - pos_component.get(me).unwrap().0;
        Action::Move {
            dx: to_target.x.signum(),
            dy: to_target.y.signum(),
        }
    }
}

#[derive(Component, Debug)]
struct AIComponent(pub Box<dyn AI>);

struct InitiativeSystem;

impl<'a> System<'a> for InitiativeSystem {
    type SystemData = (
        WriteStorage<'a, Initiative>,
        WriteStorage<'a, Ready>,
        Entities<'a>,
    );

    fn run(&mut self, (mut initiative, mut turn, entities): Self::SystemData) {
        turn.clear();
        for (initiative, entity) in (&mut initiative, &entities).join() {
            if initiative.tick() {
                turn.insert(entity, Ready)
                    .expect("can't set Ready component");
                info!("entity {:?} is ready", entity);
            }
        }
    }
}

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
        world.register::<Initiative>();
        world.register::<Ready>();
        world.register::<AIComponent>();
        Engine {
            world,
            player_action: None,
        }
    }

    pub fn set_action(&mut self, action: Action) {
        self.player_action = Some(action);
    }

    pub fn perform(&self, entity: Entity, action: &Action) {
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

    pub fn tick(&mut self) {
        let player = self.world.read_resource::<PlayerId>().0;
        let mut event_log = self.world.fetch_mut::<event_log::EventLog>();
        let mut ready = self.world.write_storage::<Ready>();
        if ready.get(player).is_some() {
            if let Some(player_action) = &self.player_action.take() {
                self.perform(player, player_action);
                event_log.log(event_log::Event::Other("stuff happened!".to_string()));
                ready.remove(player);
            } else {
                return;
            }
        }
        let mut ai = self.world.write_storage::<AIComponent>();
        let entity = self.world.entities();
        for (_ready, ai, entity) in (&ready, &mut ai, &entity).join() {
            let action = ai.0.decide(&self.world, entity);
            self.perform(entity, &action);
        }
        // Make sure `ready` is out of scope when we run the initiative system.
        drop(ready);
        InitiativeSystem.run_now(&self.world);
    }
}

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
            .with(Position(Point { x: 5, y: 5 }))
            .with(Visible {
                tile_id: TileId::Player,
            })
            .with(Initiative::new(10))
            .build();
        state
            .world
            .create_entity()
            .with(Position(Point { x: 0, y: 0 }))
            .with(Visible {
                tile_id: TileId::Grunt,
            })
            .with(Initiative::new(20))
            .with(AIComponent(Box::new(Swarm { target: player })))
            .build();
        state.world.insert(PlayerId(player));
        state.world.insert(event_log::EventLog::new());

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
        self.log_renderer.render(&event_log, window)?;

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
    // Setting min_size and max_size here tells i3 that this wants to be a floating window.
    let settings = Settings {
        min_size: Some(size),
        max_size: Some(size),
        ..Settings::default()
    };
    run::<Iterativ>("Draw Geometry", size, settings);
}
