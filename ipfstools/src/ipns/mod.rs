#![allow(dead_code)]
use crate::error::Error;
use crate::path::{IpfsPath, PathRoot};
use crate::repo::{Repo, RepoTypes};
use std::future::Future;

mod dns;
mod entry;
mod ipns_pb;

pub struct Ipns<Types: RepoTypes> {
    repo: Repo<Types>,
}

impl<Types: RepoTypes> Ipns<Types> {
    pub fn new(repo: Repo<Types>) -> Self {
        Ipns {
            repo
        }
    }

    /// Resolves a ipns path to an ipld path.
    pub fn resolve(&self, path: &IpfsPath) ->
    impl Future<Output=Result<IpfsPath, Error>>
    {
        let path = path.to_owned();
        async move {
            match path.root() {
                PathRoot::Ipld(_) => Ok(path),
                PathRoot::Dns(domain) => {
                    Ok(await!(dns::resolve(domain)?)?)
                },
                _ => Ok(path),
            }
        }
    }

    /// Publishes an ipld path.
    pub fn publish(&self, path: &IpfsPath) ->
    impl Future<Output=Result<IpfsPath, Error>>
    {
        let path = path.to_owned();
        async move {
            match path.root() {
                PathRoot::Ipld(_) => Ok(path),
                PathRoot::Dns(domain) => {
                    Ok(await!(dns::resolve(domain)?)?)
                },
                _ => Ok(path),

            }
        }
    }
}
