use macroquad::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
use native as platform;
#[cfg(target_arch = "wasm32")]
use wasm as platform;

pub use platform::*;

impl BackgroundLoader {
    pub async fn load_image(&mut self, path: &str) -> Result<Image, macroquad::Error> {
        let file = self.load_file(path).await?;
        Image::from_file_with_format(file.as_slice(), None)
    }

    pub async fn load_texture(&mut self, path: &str) -> Result<Texture2D, macroquad::Error> {
        let image = self.load_image(path).await?;
        Ok(Texture2D::from_image(&image))
    }

    pub async fn load_string(&mut self, path: &str) -> Result<String, macroquad::Error> {
        let file = self.load_file(path).await?;
        Ok(String::from_utf8_lossy(&file).to_string())
    }
}
