// Copyright 2017 rust-ipfs-api Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//

#[cfg(feature = "actix")]
extern crate actix_web;
extern crate futures;
#[cfg(feature = "hyper")]
extern crate hyper;
extern crate ipfs_api;

use futures::Future;
use ipfs_api::IpfsClient;

// Lists clients in bootstrap list, then adds the default list, then removes
// them, and readds them.
//
fn main() {
    println!("connecting to localhost:5001...");

    let client = IpfsClient::default();

    let bootstrap = client.bootstrap_list().map(|bootstrap| {
        println!("current bootstrap peers:");
        for peer in bootstrap.peers {
            println!("  {}", peer);
        }
    });

    let drop = client.bootstrap_rm_all().map(|drop| {
        println!("dropped:");
        for peer in drop.peers {
            println!("  {}", peer);
        }
    });

    let add = client.bootstrap_add_default().map(|add| {
        println!("added:");
        for peer in add.peers {
            println!("  {}", peer);
        }
    });

    let fut = bootstrap
        .and_then(|_| {
            println!();
            println!("dropping all bootstrap peers...");

            drop
        })
        .and_then(|_| {
            println!();
            println!("adding default peers...");

            add
        })
        .map_err(|e| eprintln!("{}", e));

    #[cfg(feature = "hyper")]
    hyper::rt::run(fut);
    #[cfg(feature = "actix")]
    actix_web::actix::run(|| {
        fut.then(|_| {
            actix_web::actix::System::current().stop();
            Ok(())
        })
    });
}
