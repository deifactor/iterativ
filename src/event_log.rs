use crate::components::Name;
use log::*;
use quicksilver::{
    geom::Rectangle,
    graphics::{Color, FontRenderer, Graphics},
    Result,
};
use specs::prelude::*;
use std::collections::VecDeque;

/// Something that happened that should be displayed in the game's log.
#[derive(Clone, Debug)]
pub enum Event {
    Damage {
        from: Entity,
        to: Entity,
        amount: i32,
    },
    Death {
        who: Entity,
    },
    Other(String),
}

impl Event {
    pub fn format(&self, names: &ReadStorage<Name>) -> String {
        let lookup = |who: &Entity| {
            names
                .get(*who)
                .map_or_else(|| "an unnamed bug".to_string(), |name| name.name.clone())
        };
        match self {
            Event::Damage { from, to, amount } => format!(
                "{} hits {} for {} damage.",
                lookup(from),
                lookup(to),
                amount
            ),
            Event::Death { who } => format!("{} dies.", lookup(who)),
            Event::Other(message) => message.clone(),
        }
    }
}

#[derive(Debug)]
pub struct EventLog {
    // The list of events. The head of the queue is the oldest event, and the tail is the newest
    // one.
    events: VecDeque<String>,
    capacity: usize,
}

impl EventLog {
    pub fn new() -> Self {
        EventLog {
            events: VecDeque::new(),
            capacity: 1000,
        }
    }

    pub fn log(&mut self, event: String) {
        info!("Logging event: {:?}", event);
        self.events.push_back(event);
    }

    /// Returns the events, from *newest to oldest*. This is the order that they should be rendered
    /// in.
    pub fn events(&self) -> impl Iterator<Item = &str> {
        self.events.iter().map(|s| s.as_str()).rev()
    }
}

pub struct EventLogRenderer {
    bounds: Rectangle,
    renderer: FontRenderer,
}

impl EventLogRenderer {
    pub fn new(bounds: Rectangle, renderer: FontRenderer) -> Self {
        Self { bounds, renderer }
    }

    pub fn render(
        &mut self,
        log: &EventLog,
        names: &ReadStorage<Name>,
        graphics: &mut Graphics,
    ) -> Result<()> {
        let mut lines: Vec<_> = log.events().take(5).collect();
        lines.reverse();
        let joined = lines.join("\n");
        self.renderer
            .draw_wrapping(graphics, &joined, None, Color::WHITE, self.bounds.pos)?;
        Ok(())
    }
}
