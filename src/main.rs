mod ai;
mod components;
mod engine;
mod event_log;
mod geometry;
mod map;
mod systems;
mod tiles;

use crate::ai::{AIComponent, PlayerAI, Swarm};
use crate::components::*;
use crate::engine::*;
use crate::event_log::EventLogRenderer;
use crate::tiles::*;
use quicksilver::{
    geom::{Rectangle, Vector},
    graphics::{Color, Graphics, VectorFont},
    input::{Event, Input, Key},
    Result, Settings, Window,
};
use specs::prelude::*;

const WIDTH: i32 = 80;
const HEIGHT: i32 = 40;
const MAP_HEIGHT: i32 = 30;
const TILE_SIZE: i32 = 16;

struct Iterativ {
    tiles: Tiles,
    state: Engine,
    window: Window,
    graphics: Graphics,
    log_renderer: EventLogRenderer,
}

impl Iterativ {
    async fn new(window: Window, graphics: Graphics) -> quicksilver::Result<Iterativ> {
        let tiles = Tiles::new(
            &graphics,
            "sprites",
            (TILE_SIZE as f32, TILE_SIZE as f32).into(),
        )
        .await?;
        let mut state = Engine::new();
        let player = state
            .world
            .create_entity()
            .with(Name {
                name: "you".to_string(),
            })
            .with(Position((5, 5).into()))
            .with(AIComponent(Box::new(PlayerAI)))
            .with(IsPlayer)
            .with(Visible {
                tile_id: TileId::Player,
            })
            .with(CombatStats {
                max_hp: 20,
                hp: 20,
                attack: 3,
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
            .with(Position((0, 0).into()))
            .with(Visible {
                tile_id: TileId::Grunt,
            })
            .with(CombatStats {
                max_hp: 5,
                hp: 5,
                attack: 1,
            })
            .with(Initiative::new(20))
            .with(BlocksMovement)
            .with(AIComponent(Box::new(Swarm { target: player })))
            .build();
        state.world.insert(PlayerId(player));
        state.world.insert(event_log::EventLog::new());
        state.world.insert(map::Map::new(WIDTH, HEIGHT));

        let font = VectorFont::from_bytes(include_bytes!("../static/white_rabbit.ttf").to_vec());
        let renderer = font.to_renderer(&graphics, 16.0)?;
        let log_renderer = EventLogRenderer::new(
            Rectangle::new(
                Vector {
                    x: 0.0,
                    y: (MAP_HEIGHT * TILE_SIZE) as f32,
                },
                Vector {
                    x: (WIDTH * TILE_SIZE) as f32,
                    y: ((HEIGHT - MAP_HEIGHT) * TILE_SIZE) as f32,
                },
            ),
            renderer,
        );
        Ok(Iterativ {
            window,
            graphics,
            tiles,
            state,
            log_renderer,
        })
    }

    fn draw(&mut self) -> Result<()> {
        self.graphics.clear(Color::BLACK);

        let positions = self.state.world.read_storage::<Position>();
        let visibles = self.state.world.read_storage::<Visible>();

        let graphics = &mut self.graphics;

        for (pos, vis) in (&positions, &visibles).join() {
            // XXX: get rid of these magic constants.
            let vec = Vector {
                x: 16.0 * (pos.0.x as f32),
                y: 16.0 * (pos.0.y as f32),
            };
            graphics.draw_image(
                &self.tiles.tile(vis.tile_id),
                Rectangle::new(vec, self.tiles.size()),
            );
        }

        let event_log = self.state.world.fetch::<event_log::EventLog>();
        let names = self.state.world.read_storage::<Name>();
        self.log_renderer
            .render(&event_log, &names, &mut self.graphics)?;

        self.graphics.present(&self.window)?;
        Ok(())
    }

    pub fn event(&mut self, event: &Event) -> Result<()> {
        if self.state.loop_state() == LoopState::GameOver {
            // disable game input in game over
            return Ok(());
        }
        if let Event::KeyboardInput(ev) = event {
            if !ev.is_down() {
                return Ok(());
            }
            match ev.key() {
                Key::H => self.state.set_action(Action::Move {
                    motion: (-1, 0).into(),
                }),
                Key::J => self.state.set_action(Action::Move {
                    motion: (0, 1).into(),
                }),
                Key::K => self.state.set_action(Action::Move {
                    motion: (0, -1).into(),
                }),
                Key::L => self.state.set_action(Action::Move {
                    motion: (1, 0).into(),
                }),
                Key::Y => self.state.set_action(Action::Move {
                    motion: (-1, -1).into(),
                }),
                Key::U => self.state.set_action(Action::Move {
                    motion: (1, -1).into(),
                }),
                Key::B => self.state.set_action(Action::Move {
                    motion: (-1, 1).into(),
                }),
                Key::N => self.state.set_action(Action::Move {
                    motion: (1, 1).into(),
                }),
                _ => (),
            }
        }
        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        match self.state.loop_state() {
            LoopState::WaitingForPlayer | LoopState::Looping => {
                self.state.tick();
                self.state.world.maintain();
            }
            LoopState::GameOver => {}
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
fn wasm_panic_hook(info: &std::panic::PanicInfo) {
    use stdweb::console;
    console!(error, info.to_string());
}

async fn app(window: Window, gfx: Graphics, mut input: Input) -> Result<()> {
    let mut app = Iterativ::new(window, gfx).await?;
    loop {
        while let Some(event) = input.next_event().await {
            app.event(&event)?
        }
        app.update()?;
        app.draw()?;
    }
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    std::panic::set_hook(Box::new(wasm_panic_hook));

    let size = Vector::new((WIDTH * TILE_SIZE) as f32, (HEIGHT * TILE_SIZE) as f32);
    // Setting min_size and max_size here tells i3 that this wants to be a floating window. the
    let settings = Settings {
        size,
        vsync: true,
        ..Settings::default()
    };
    quicksilver::run(settings, app);
}
