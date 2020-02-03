use crate::engine::Name;
use log::*;
use quicksilver::prelude::*;
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
                .map_or_else(|| "oops!".to_string(), |name| name.name.clone())
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
    events: VecDeque<Event>,
    capacity: usize,
}

impl EventLog {
    pub fn new() -> Self {
        EventLog {
            events: VecDeque::new(),
            capacity: 1000,
        }
    }

    pub fn log(&mut self, event: Event) {
        info!("Logging event: {:?}", event);
        self.events.push_back(event);
    }

    /// Returns the events, from *newest to oldest*. This is the order that they should be rendered
    /// in.
    pub fn events(&self) -> impl Iterator<Item = &Event> {
        self.events.iter().rev()
    }
}

pub struct EventLogRenderer {
    bounds: Rectangle,
    font: Font,
    style: FontStyle,
    /// The number of characters that we can draw.
    character_bounds: (i32, i32),
}

impl EventLogRenderer {
    pub fn new(bounds: Rectangle, font: Font, style: FontStyle) -> Result<Self> {
        let font_size = font.render(".", &style)?.area().size();
        let character_bounds = (
            (bounds.width() / font_size.x) as i32,
            (bounds.height() / font_size.y) as i32,
        );
        info!(
            "Setting up renderer with bounds {:?}, character bounds {:?}",
            bounds, character_bounds
        );
        Ok(EventLogRenderer {
            bounds,
            font,
            style,
            character_bounds,
        })
    }

    pub fn render(
        &self,
        log: &EventLog,
        names: &ReadStorage<Name>,
        window: &mut Window,
    ) -> Result<()> {
        let (char_width, char_height) = self.character_bounds;
        // Get the `char_height` most recent events, from oldest to newest.
        let mut lines: Vec<_> = log
            .events()
            .map(|ev| ev.format(names))
            .take(char_height as usize)
            .collect();
        lines.reverse();

        let wrapped_lines = textwrap::fill(&lines.join("\n"), char_width as usize);
        let rendered = self.font.render(&wrapped_lines, &self.style)?;
        let y_coord = self.bounds.y() + self.bounds.height() - rendered.area().height();
        let pos = Rectangle::new((self.bounds.x(), y_coord), rendered.area().size());
        window.draw(&pos, Img(&rendered));
        Ok(())
    }
}
