// Copyright 2017 rust-ipfs-api Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use response::serde;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PubsubLsResponse {
    #[serde(deserialize_with = "serde::deserialize_vec")]
    pub strings: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PubsubPeersResponse {
    #[serde(deserialize_with = "serde::deserialize_vec")]
    pub strings: Vec<String>,
}

pub type PubsubPubResponse = ();

#[derive(Debug, Deserialize)]
pub struct PubsubSubResponse {
    pub from: Option<String>,
    pub data: Option<String>,
    pub seqno: Option<String>,

    #[serde(rename = "topicIDs")]
    pub topic_ids: Option<Vec<String>>,

    #[serde(rename = "XXX_unrecognized")]
    pub unrecognized: Option<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    deserialize_test!(v0_pubsub_ls_0, PubsubLsResponse);
    deserialize_test!(v0_pubsub_ls_1, PubsubLsResponse);
    deserialize_test!(v0_pubsub_peers_0, PubsubPeersResponse);
    deserialize_test!(v0_pubsub_sub_0, PubsubSubResponse);
    deserialize_test!(v0_pubsub_sub_1, PubsubSubResponse);
}
