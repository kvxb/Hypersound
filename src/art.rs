use image::{imageops::FilterType, RgbaImage};
use lofty::file::TaggedFileExt;
use std::path::{Path, PathBuf};

pub struct ArtLoader {
    loaded_for: Option<PathBuf>,
    loaded_size: (u32, u32),
    image: Option<RgbaImage>,
}

impl ArtLoader {
    pub fn new() -> Self {
        Self {
            loaded_for: None,
            loaded_size: (0, 0),
            image: None,
        }
    }

    pub fn get(&mut self, path: Option<&Path>, width: u32, height: u32) -> Option<&RgbaImage> {
        let path = path?;
        let size = (width, height);
        if self.loaded_for.as_deref() == Some(path) && self.loaded_size == size {
            return self.image.as_ref();
        }
        self.loaded_for = Some(path.to_path_buf());
        self.loaded_size = size;
        self.image = load(path, width, height);
        self.image.as_ref()
    }
}

fn load(path: &Path, width: u32, height: u32) -> Option<RgbaImage> {
    let tagged = lofty::read_from_path(path).ok()?;
    let tag = tagged.primary_tag().or_else(|| tagged.tags().first())?;
    let picture = tag.pictures().first()?;
    let decoded = image::load_from_memory(picture.data()).ok()?;
    Some(
        decoded
            .resize_exact(width, height, FilterType::Lanczos3)
            .to_rgba8(),
    )
}
