// Copyright 2017 rust-filesys-api Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TarAddResponse {
    pub name: String,
    pub hash: String,
}

#[cfg(test)]
mod tests {
    deserialize_test!(v0_tar_add_0, TarAddResponse);
}
