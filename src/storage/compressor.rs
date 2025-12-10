use anyhow::{Context, Ok};
use caesium::{compress_in_memory, parameters::CSParameters};

#[derive(Clone, Default)]
pub struct Compressor {
    parameters: CSParameters,
}
impl Compressor {
    pub fn with_defaults() -> Self {
        Default::default()
    }
    pub fn with_sane_defaults() -> Self {
        let quality = 80;
        let mut params = CSParameters::new();
        params.keep_metadata = false;
        params.jpeg.quality = quality;
        params.png.quality = quality;
        params.webp.quality = quality;
        Self { parameters: params }
    }
    pub fn compress(&self, payload: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        let res = compress_in_memory(payload, &self.parameters).context("Error comprimiendo")?;
        Ok(res)
    }
}
