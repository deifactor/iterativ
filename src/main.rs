mod ai;
mod event_log;
mod geometry;
mod map;
mod tiles;

use crate::ai::{AIComponent, PlayerAI, PlayerAction, Swarm};
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

struct InitiativeSystem;

impl<'a> System<'a> for InitiativeSystem {
    type SystemData = (
        WriteStorage<'a, Initiative>,
        WriteStorage<'a, Ready>,
        ReadExpect<'a, LoopState>,
        Entities<'a>,
    );

    fn run(&mut self, (mut initiative, mut turn, loop_state, entities): Self::SystemData) {
        if *loop_state != LoopState::Looping {
            return;
        }

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

#[derive(Debug, Clone)]
pub enum Action {
    Move { dx: i32, dy: i32 },
}

#[derive(Component, Default, Debug, Copy, Clone)]
/// Tagged to indicate that an entity cannot be moved through.
struct BlocksMovement;

struct MapUpdateSystem;

impl<'a> System<'a> for MapUpdateSystem {
    type SystemData = (
        WriteExpect<'a, map::Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksMovement>,
        Entities<'a>,
    );

    fn run(&mut self, (mut map, positions, blocked, entities): Self::SystemData) {
        map.clear_entities();
        for (position, blocked, entity) in (&positions, blocked.maybe(), &entities).join() {
            let idx = map.idx(position.0.x, position.0.y);
            if blocked.is_some() {
                map.blocked_by_entity[idx] = true;
            }
            map.entities[idx].push(entity);
        }
    }
}

#[derive(Component, Debug, Clone)]
/// If something should be drawn in the world, what's its tile? This is *not* the underlying Image,
/// since that's wrapped in an Rc and so it's not thread-safe.
struct Visible {
    pub tile_id: TileId,
}

pub struct Engine {
    pub world: World,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum LoopState {
    Looping,
    WaitingForPlayer,
}

impl Engine {
    pub fn new() -> Self {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<BlocksMovement>();
        world.register::<Visible>();
        world.register::<Initiative>();
        world.register::<Ready>();
        world.register::<ai::AIComponent>();
        world.insert(LoopState::Looping);
        Engine { world }
    }

    /// Indicates that the player has decided on what they want to do.
    pub fn set_action(&mut self, action: Action) {
        self.world.insert(PlayerAction(action))
    }

    pub fn perform(&mut self, entity: Entity, action: &Action) {
        info!("{:?} performing {:?}", entity, action);
        if entity == self.world.fetch::<PlayerId>().0 {
            // Clear out the player's action, since we're about to execute it.
            self.world.remove::<PlayerAction>();
            // We're no longer waiting for the player.
            self.world.insert::<LoopState>(LoopState::Looping);
        }
        match action {
            Action::Move { dx, dy } => {
                let map = self.world.fetch::<map::Map>();
                let mut pos_storage = self.world.write_storage::<Position>();
                let pos = pos_storage
                    .get_mut(entity)
                    .expect("can't move something without a position");
                let new_pos = (pos.0.x + dx, pos.0.y + dy);
                if !map.blocked(new_pos.0, new_pos.1) {
                    pos.0.x += dx;
                    pos.0.y += dy;
                }
            }
        }
    }

    fn find_actor(&self) -> Option<(Entity, Action)> {
        let ready = self.world.write_storage::<Ready>();
        let mut ai = self.world.write_storage::<ai::AIComponent>();
        let entity = self.world.entities();
        let player = self.world.fetch::<PlayerId>().0;
        for (_ready, ai, entity) in (&ready, &mut ai, &entity).join() {
            if let Some(action) = ai.0.decide(&self.world, entity) {
                return Some((entity, action));
            } else if entity == player {
                *self.world.fetch_mut::<LoopState>() = LoopState::WaitingForPlayer;
                return None;
            }
        }
        None
    }

    // The game loop works like this: we find an entity that has something that it can do. If at
    // any point the player is ready but doesn't have anything it can do, we enter the 'waiting for
    // player' state. In this state, the initiative system does nothing. And even if there are mobs
    // that are also ready, we'll eventually execute their actions given enough runs through
    // tick(), meaning that we'll get into a situation where the player is the only mob that's
    // ready, but it's not doing anything; i.e., we're waiting for playre input.
    //
    // Then, when the player inputs something, the next tick through will finally have the player's
    // "AI" return an action, causing us to leave `WaitingForPlayer` and resume normal engine
    // execution.

    pub fn tick(&mut self) {
        InitiativeSystem.run_now(&self.world);
        if let Some((entity, action)) = self.find_actor() {
            self.world.write_storage::<Ready>().remove(entity);
            self.perform(entity, &action);
            MapUpdateSystem.run_now(&self.world);
        }
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
