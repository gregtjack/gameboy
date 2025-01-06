use anyhow::Result;
use std::path::PathBuf;

pub fn read(path: PathBuf) -> Result<Vec<u8>> {
    let bytes = std::fs::read(path)?;
    Ok(bytes)
}
