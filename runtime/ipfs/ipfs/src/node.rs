use cid::Cid;
use crate::block::Block;
use crate::core::Resolver;
use crate::block::BlockTrait;
use std::error::Error;

pub struct Link {
    name: String,
    size: u64,
    cid: Cid,
}

// NodeStat is a statistics object for a Node. Mostly sizes.
#[derive(Debug)]
pub struct NodeStat {
    hash: String,
    // number of links in link table
    num_links: u64,
    // size of the raw, encoded data
    block_size: u64,
    // size of the links segment
    links_size: u64,
    // size of the data segment
    data_size: u64,
    // cumulative size of object and its references
    cumulative_size: u64,
}

//consider remove inheritance relationship, I have no idea but troublesome
pub trait NodeTrait: Resolver + Clone + BlockTrait {
    fn resolve_link(&self, path: Vec<String>) -> Result<(Link, Vec<String>), Box<Error>>;

    fn links(&self) -> Vec<Link>;

    fn stat(&self) -> Result<NodeStat, Box<Error>>;

    fn size(&self) -> Result<u64, Box<Error>>;
}

pub struct Node<T: NodeTrait + Sized> {
    node: Box<T>,
}

impl<T: NodeTrait> Node<T> {
    fn make_link(&self) -> Link {
        let size = self.node.size().unwrap();

        let cid = self.node.cid();
        Link {
            name: String::new(),
            size,
            cid,
        }
    }
}