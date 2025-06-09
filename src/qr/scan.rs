use bardecoder::Decoder;
use image::{GrayImage, RgbaImage};

pub struct Scanner {
    pub decoder: Decoder<RgbaImage, GrayImage, String>,
    pub image: RgbaImage,
}
impl Default for Scanner {
    fn default() -> Self {
        let image = RgbaImage::new(400, 240);
        let decoder = bardecoder::default_decoder();
        Self { decoder, image }
    }
}
impl Scanner {
    pub fn scan(&mut self) -> anyhow::Result<()> {
        let mut decoded = self.decoder.decode(&self.image);
        println!("{decoded:?}");
        while let Some(Ok(frame)) = decoded.pop() {
            println!("found frame! {frame}");
        }
        Ok(())
    }
}
