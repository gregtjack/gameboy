use std::path::PathBuf;
use anyhow::Result;

pub fn read(path: PathBuf) -> Result<Vec<u8>> {
    let bytes = std::fs::read(path)?;
    Ok(bytes)
}
