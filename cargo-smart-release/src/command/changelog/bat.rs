use std::{io, path::Path, process::Command};

pub struct Support {
    bat_found: bool,
}

impl Support {
    pub fn new() -> Self {
        Support {
            bat_found: Command::new("bat").arg("--version").output().is_ok(),
        }
    }

    pub fn display_to_tty(&self, path: &Path) -> io::Result<()> {
        if !self.bat_found {
            log::info!("Would want to use 'bat' for colored preview, but it wasn't available in the PATH.");
            return Ok(());
        }
        if Command::new("bat")
            .args(&["--paging=always", "-l=md"])
            .arg(path)
            .status()?
            .success()
        {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "bat exited with an error"))
        }
    }
}
