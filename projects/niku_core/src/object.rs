// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0



//! Structs and enums related to the concept of an object.

use std::fmt::{Debug, Display};

use iroh::NodeAddr;
use iroh_blobs::Hash;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// The kind of an object
pub enum ObjectKind {
    /// The object is a file
    File {
        /// The name of the file
        name: String,
    },

    /// The object is a folder
    Folder {
        /// The name of the folder
        name: String,
    },
}

impl Display for ObjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            ObjectKind::File { name: _ } => "file",
            ObjectKind::Folder { name: _ } => "folder",
        };

        write!(f, "{}", text)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Entry that holds the relevant data of a object available for downloading.
pub struct ObjectEntry {
    /// The [iroh] address of the node that is hosting the file.
    pub node_address: NodeAddr,

    /// The file hash used by [iroh_blobs] protocol to access the file.
    pub file_hash: Hash,

    /// The kind of object to be download.
    pub kind: ObjectKind,

    /// The number of bytes of the object.
    pub size: u64,
}
