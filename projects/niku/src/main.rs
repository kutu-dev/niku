// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

use log::error;
use niku::run;
use niku_core::set_cli_logging;

#[tokio::main]
async fn main() {
    set_cli_logging();

    match run().await {
        Ok(_) => (),
        Err(err) => error!("{err}"),
    }
}
