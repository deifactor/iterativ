use quicksilver::prelude::*;

pub struct Tiles {
    pub player: Image,
}

impl Tiles {
    pub fn render(font: &Font) -> Result<Self> {
        let player = font.render("@", &FontStyle::new(14.0, Color::BLACK))?;
        Ok(Tiles { player })
    }
}
