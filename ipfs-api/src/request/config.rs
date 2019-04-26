// Copyright 2017 rust-ipfs-api Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

use http::Method;
use request::ApiRequest;

pub struct ConfigEdit;

impl_skip_serialize!(ConfigEdit);

impl ApiRequest for ConfigEdit {
    const PATH: &'static str = "/config/edit";
}

pub struct ConfigReplace;

impl_skip_serialize!(ConfigReplace);

impl ApiRequest for ConfigReplace {
    const PATH: &'static str = "/config/replace";

    const METHOD: &'static Method = &Method::POST;
}

pub struct ConfigShow;

impl_skip_serialize!(ConfigShow);

impl ApiRequest for ConfigShow {
    const PATH: &'static str = "/config/show";
}
