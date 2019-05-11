use std::any::Any;
use std::error::Error;
use crate::block::Block;
use crate::node::NodeTrait;
use crate::node::Node;
use std::collections::HashMap;


pub trait Resolver {
    fn resolve<T: Any + Sized>(&self, path: Vec<String>) -> Result<(T, Vec<String>), Box<Error>>;

    fn tree(&self, path: String, depth: u32) -> Vec<String>;
}

//todo define our own error struct, and return Result<Block,MyError>
type DecodeBlockFn<T> = fn(Block) -> Node<T>;

pub trait BlockDecoder<T: NodeTrait> {
    fn register(&mut self, codec: u64, decoder: DecodeBlockFn<T>);
    fn decode(&self, block: Block) -> Node<T>;
}

pub struct SafeBlockDecoder<T: NodeTrait> {
    decoders: HashMap<u64, DecodeBlockFn<T>>
}

impl<T: NodeTrait> BlockDecoder<T> for SafeBlockDecoder<T> {
    fn register(&mut self, codec: u64, decoder: fn(Block) -> Node<T>) {
        //todo thread safe
        self.decoders.insert(codec, decoder);
    }

    fn decode(&self, block: Block) -> Node<T> {
        let codec = block.block.cid().codec.into();
        let decoder = self.decoders.get(&codec).unwrap();
        decoder(block)
    }
}


#[cfg(test)]
mod tests {
    use super::NodeTrait;
    use crate::node::Link;
    use crate::node::NodeStat;
    use std::error::Error;
    use crate::core::SafeBlockDecoder;
    use std::collections::HashMap;
    use crate::core::DecodeBlockFn;
    use crate::block::BlockTrait;
    use cid::Cid;
    use crate::core::Resolver;
    use std::any::Any;
    use crate::core::BlockDecoder;
    use crate::block::Block;
    use crate::node::Node;


    #[derive(Clone, Debug)]
    pub struct MyNode {}

    impl ToString for MyNode {
        fn to_string(&self) -> String {
            unimplemented!()
        }
    }

    impl Resolver for MyNode {
        fn resolve<T: Any + Sized>(&self, path: Vec<String>) -> Result<(T, Vec<String>), Box<Error>> {
            unimplemented!()
        }

        fn tree(&self, path: String, depth: u32) -> Vec<String> {
            unimplemented!()
        }
    }

    impl BlockTrait for MyNode {
        fn raw_data(&self) -> Vec<u8> {
            unimplemented!()
        }

        fn cid(&self) -> Cid {
            unimplemented!()
        }
    }

    impl NodeTrait for MyNode {
        fn resolve_link(&self, path: Vec<String>) -> Result<(Link, Vec<String>), Box<Error>> {
            unimplemented!()
        }

        fn links(&self) -> Vec<Link> {
            unimplemented!()
        }

        fn stat(&self) -> Result<NodeStat, Box<Error>> {
            unimplemented!()
        }

        fn size(&self) -> Result<u64, Box<Error>> {
            unimplemented!()
        }
    }

    fn f(block: Block) -> Node<MyNode> {
        unimplemented!()
    }

    #[test]
    fn it_works() {
        let hash_map: HashMap<u64, DecodeBlockFn<MyNode>> = HashMap::new();
        let mut safe_block_decoder = SafeBlockDecoder {
            decoders: hash_map
        };


        safe_block_decoder.register(1, f);
    }
}