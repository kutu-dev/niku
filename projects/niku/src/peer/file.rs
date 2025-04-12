use std::path::PathBuf;

use anyhow::Result;
use iroh_blobs::rpc::client::blobs::WrapOption;
use iroh_blobs::util::SetTagOption;

use super::{Peer, PeerError};
use crate::object::{HashWrapper, NodeAddrWrapper, ObjectEntry, ObjectKind};

impl Peer {
    /// Creates a new object entry for a file.
    ///
    /// # Safety
    /// Doesn't check if the given path is for a file.
    pub async unsafe fn create_file_object_entry(
        &mut self,
        path: PathBuf,
    ) -> Result<ObjectEntry, PeerError> {
        let blob = self
            .blobs
            .client()
            .add_from_path(path.clone(), true, SetTagOption::Auto, WrapOption::NoWrap)
            .await?
            .finish()
            .await?;

        #[allow(clippy::expect_used)]
        let file_name = path
            .file_name()
            .expect("The path is always for a real file")
            .to_str()
            .ok_or(PeerError::NotUnicodePath)?
            .to_string();

        Ok(ObjectEntry {
            node_address: NodeAddrWrapper(self.router.endpoint().node_addr().await?),
            file_hash: HashWrapper(blob.hash),
            kind: ObjectKind::File { name: file_name },
            size: blob.size,
        })
    }
}
