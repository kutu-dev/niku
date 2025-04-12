//! Code that implements a NIKU peer.

mod file;
mod folder;
mod request;

use std::io;

use anyhow::Result;
use iroh::protocol::Router;
use iroh::Endpoint;
use iroh_blobs::net_protocol::Blobs;
use log::debug;
use reqwest::Method;
use serde_json::error;
use thiserror::Error;
use zip::result::ZipError;

use crate::backend::{ErrorResponse, ObjectKeepAliveRequest, RegisteredObjectData};
use crate::object::ObjectEntry;

/// Peer used to interact with other NIKU clients.
pub struct Peer {
    client: reqwest::Client,
    blobs: Blobs<iroh_blobs::store::mem::Store>,
    router: Router,
}

/// Errors that may happen when interacting with an NIKU peer.
#[derive(Debug, Error)]
pub enum PeerError {
    /// An error from Iroh.
    #[error("An error from Iroh has been raised: {0}")]
    IrohError(#[from] anyhow::Error),

    /// The given path is not encoded with UTF-8 (Unicode)
    #[error("The given path is not encoded with UTF-8 (Unicode)")]
    NotUnicodePath,

    /// Unable to send an object to the backend server.
    #[error("Unable to send the object to the backend server: {0}")]
    PublishObjectFailed(#[source] reqwest::Error),

    /// Unable to send an object to the backend server due to a malformed response.
    #[error("Unable to send the object to the backend server  due to a malformed response.: {0}")]
    MalformedResponse(#[from] serde_json::Error),

    /// An error from the backend.
    #[error("The backend returned the error: {0}")]
    BackendError(#[from] ErrorResponse),

    /// Unable to write into the filesystem.
    #[error("Unable to write into the filesystem: {0}")]
    UnableToWritoIntoTheFilesystem(#[source] io::Error),

    #[error("Unable to finish the compression: {0}")]
    CompressionError(#[from] ZipError),

    #[error("The given folder is the root")]
    FolderIsRoot,

    #[error("Unable to strip the prefix of a path: {0}")]
    StripPrefixError(#[from] std::path::StripPrefixError),

    #[error("The current working directory is not valid: {0}")]
    CurrentWorkingDirectoryInvalid(#[source] io::Error),

    #[error("Unable to open a file: {0}")]
    UnableToOpenAFile(#[source] io::Error),

    #[error("Unable to create a directory: {0}")]
    UnableToCreateADirectory(#[source] io::Error),
}

impl Peer {
    /// Make a new [Peer].
    pub async fn new() -> Result<Peer, PeerError> {
        let client = reqwest::Client::new();
        let endpoint = Endpoint::builder().bind().await?;

        let blobs = Blobs::memory().build(&endpoint);

        let router = Router::builder(endpoint)
            .accept(iroh_blobs::ALPN, blobs.clone())
            .spawn()
            .await?;

        Ok(Peer {
            client,
            blobs,
            router,
        })
    }

    /// Safetly shutdown the peer.
    pub async fn async_drop(self) -> Result<(), PeerError> {
        debug!("Shuting down the peer...");
        Ok(self.router.shutdown().await?)
    }

    /// Publish an object entry to the most available backend server.
    pub async fn publish_object_entry(
        &self,
        object_entry: &ObjectEntry,
    ) -> Result<RegisteredObjectData, PeerError> {
        self.request_expect_json(Method::PUT, "objects", Some(object_entry))
            .await
    }

    pub async fn retrieve_object_entry(&self, id: &str) -> Result<ObjectEntry, PeerError> {
        self.request_expect_json(Method::GET, &format!("objects/{id}"), None::<&()>)
            .await
    }

    /// Keep alive the given object entry.
    pub async fn keep_alive_object_entry(
        &self,
        registered_object_entry: &RegisteredObjectData,
    ) -> Result<(), PeerError> {
        self.request(
            Method::POST,
            &format!("objects/{}/keep-alive", registered_object_entry.id),
            Some(&ObjectKeepAliveRequest {
                keep_alive_key: registered_object_entry.keep_alive_key.clone(),
            }),
        )
        .await?;

        Ok(())
    }

    /// Download an object entry into the Iroh store.
    pub async fn download_object_entry(&self, object_entry: &ObjectEntry) -> Result<(), PeerError> {
        self.blobs
            .client()
            .download(
                object_entry.file_hash.0,
                object_entry.node_address.0.clone(),
            )
            .await?
            .finish()
            .await?;

        Ok(())
    }
}
