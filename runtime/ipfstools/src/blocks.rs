use cid::Cid;
use multihash::Multihash;
use multihash::decode;

pub trait BlockTrait: ToString {
    fn raw_data(&self) -> Vec<u8>;

    fn cid(&self) -> Cid;
}

pub struct Block {
    pub block: Box<BlockTrait>
}

pub struct BasicBlock {
    cid: Cid,
    data: Vec<u8>,
}

impl BlockTrait for BasicBlock {
    fn raw_data(&self) -> Vec<u8> {
        self.data.clone()
    }

    fn cid(&self) -> Cid {
        self.cid.clone()
    }
}

impl ToString for BasicBlock {
    fn to_string(&self) -> String {
        format!("block {}", self.cid.clone())
    }
}

impl BasicBlock {
    fn new_block(data: Vec<u8>) -> Self {
        BasicBlock {
            //todo check codec param
            cid: cid::Cid::new(cid::Codec::BitcoinTx, cid::Version::V0, &data),
            data,
        }
    }

    fn new_block_with_cid(data: Vec<u8>, cid: cid::Cid) -> Self {
        BasicBlock {
            cid,
            data,
        }
    }

    fn multihash(&self) -> Multihash {
        decode(&self.cid.hash).unwrap()
    }
}