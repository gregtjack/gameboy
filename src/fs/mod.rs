use color_eyre::Result;
use std::path::PathBuf;
use tracing::debug;

pub fn read(path: PathBuf) -> Result<Vec<u8>> {
    debug!("Reading bytes from path: {:?}", path);
    let bytes = std::fs::read(path)?;
    Ok(bytes)
}
