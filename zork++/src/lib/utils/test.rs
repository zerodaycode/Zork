use std::path::Path;

use color_eyre::Result;
use tempfile::tempdir;

pub fn in_temp_dir<F: FnOnce(&Path) -> Result<()>>(f: F) -> Result<()> {
    let previous_dir = std::env::current_dir()?;

    let temp = tempdir()?;
    std::env::set_current_dir(&temp)?;

    f(temp.path())?;

    std::env::set_current_dir(previous_dir)?;

    Ok(())
}
