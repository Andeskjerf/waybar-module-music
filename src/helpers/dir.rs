use std::{fs, path::PathBuf};

/// Gets PathBuf depending on given callback, then creates a new directory within that with the module's name
/// Takes a function that returns a directory, for example, `dirs::cache_dir()`
pub fn get_and_create_dir<F>(callback: F) -> Result<PathBuf, Box<dyn std::error::Error>>
where
    F: Fn() -> Option<PathBuf>,
{
    let directory = callback()
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "could not get directory")
        })?
        .join("waybar-module-music");

    let _ = fs::create_dir(&directory);

    Ok(directory)
}
