use enum_iterator::IntoEnumIterator;
use quicksilver::{
    geom::{Rectangle, Vector},
    graphics::{Graphics, Image},
    Result,
};
use std::collections::HashMap;
use std::path::Path;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, IntoEnumIterator)]
pub enum TileId {
    Player,
    Grunt,
}

impl TileId {
    fn filename(&self) -> &'static str {
        match self {
            TileId::Player => "player",
            TileId::Grunt => "grunt",
        }
    }
}

pub struct Tiles {
    size: Vector,
    // This hashmap is guaranteed to have an entry for each TileId.
    images: HashMap<TileId, Image>,
}

impl Tiles {
    /// Loads all the tiles in the given directory. This function checks that all fo the tiles have
    /// the given size.
    pub async fn new<P: AsRef<Path>>(gfx: &Graphics, directory: P, size: Vector) -> Result<Self> {
        let mut images = HashMap::new();
        for tile in TileId::into_enum_iter() {
            let filename = directory
                .as_ref()
                .join(tile.filename())
                .with_extension("png");
            let image = Image::load(gfx, filename).await?;
            assert_eq!(image.size(), size);
            images.insert(tile, image);
        }
        Ok(Self { size, images })
    }

    pub fn tile(&self, tile: TileId) -> &Image {
        &self.images[&tile]
    }

    pub fn size(&self) -> Vector {
        self.size
    }
}
