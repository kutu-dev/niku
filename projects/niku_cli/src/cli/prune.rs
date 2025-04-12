use anyhow::Result;
use log::info;

use super::{Cli, CliError};

impl Cli {
    pub(super) async fn prune() -> Result<(), CliError> {
        let cache_path = niku::get_cache_path();

        if !cache_path.exists() {
            info!("The cache is empty!");
            return Ok(());
        }

        if !cache_path.is_dir() {
            info!("The cache is not a folder! Force deleting it...");
            tokio::fs::remove_file(cache_path).await?;
            return Ok(());
        }

        let cache_size = fs_extra::dir::get_size(cache_path.clone())?;

        info!(
            "Prunning the cache ({})...",
            niku::format_bytes_with_unit(cache_size)
        );
        std::fs::remove_dir_all(cache_path)?;
        info!("Done!");

        Ok(())
    }
}
