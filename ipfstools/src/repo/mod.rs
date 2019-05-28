//! IPFS repo
use crate::block::{Cid, Block};
use crate::error::Error;
use crate::future::BlockFuture;
use crate::IpfsOptions;
use core::future::Future;
use futures::future::FutureObj;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender, Receiver};

pub mod mem;
pub mod fs;

pub trait RepoTypes: Clone + Send + Sync + 'static {
    type TBlockStore: BlockStore;
}

#[derive(Clone, Debug)]
pub struct RepoOptions<TRepoTypes: RepoTypes> {
    _marker: PhantomData<TRepoTypes>,
    path: PathBuf,
}

impl<TRepoTypes: RepoTypes> From<&IpfsOptions<TRepoTypes>> for RepoOptions<TRepoTypes> {
    fn from(options: &IpfsOptions<TRepoTypes>) -> Self {
        RepoOptions {
            _marker: PhantomData,
            path: options.ipfs_path.clone(),
        }
    }
}

pub fn create_repo<TRepoTypes: RepoTypes>(options: RepoOptions<TRepoTypes>) -> (Repo<TRepoTypes>, Receiver<RepoEvent>) {
    Repo::new(options)
}

pub trait BlockStore: Clone + Send + Sync + Unpin + 'static {
    fn new(path: PathBuf) -> Self;
    fn init(&self) ->
        FutureObj<'static, Result<(), Error>>;
    fn open(&self) ->
        FutureObj<'static, Result<(), Error>>;
    fn contains(&self, cid: &Cid) ->
        FutureObj<'static, Result<bool, Error>>;
    fn get(&self, cid: &Cid) ->
        FutureObj<'static, Result<Option<Block>, Error>>;
    fn put(&self, block: Block) ->
        FutureObj<'static, Result<Cid, Error>>;
    fn remove(&self, cid: &Cid) ->
        FutureObj<'static, Result<(), Error>>;
}

#[derive(Clone, Copy, Debug)]
pub enum Column {
    Ipns
}

#[derive(Clone, Copy, Debug)]
pub enum DBColumn {
    BeaconBlock,
    BeaconState,
    ShardBlock,
}

#[derive(Clone, Debug)]
pub struct Repo<TRepoTypes: RepoTypes> {
    block_store: TRepoTypes::TBlockStore,
    events: Sender<RepoEvent>,
}

#[derive(Clone, Debug)]
pub enum RepoEvent {
    WantBlock(Cid),
    ProvideBlock(Cid),
    UnprovideBlock(Cid),
}

impl<TRepoTypes: RepoTypes> Repo<TRepoTypes> {
    pub fn new(options: RepoOptions<TRepoTypes>) -> (Self, Receiver<RepoEvent>) {
        let mut blockstore_path = options.path.clone();
        blockstore_path.push("blockstore");
        let block_store = TRepoTypes::TBlockStore::new(blockstore_path);
        let (sender, receiver) = channel::<RepoEvent>();

        (Repo {
            block_store,
            events: sender,
        }, receiver)
    }

    pub fn init(&self) -> FutureObj<'static, Result<(), Error>> {
        self.block_store.init()
    }

    pub fn open(&self) -> FutureObj<'static, Result<(), Error>> {
        self.block_store.open()
    }

    /// Puts a block into the block store.
    pub fn put_block(&self, block: Block) ->
    impl Future<Output=Result<Cid, Error>>
    {
        let events = self.events.clone();
        let block_store = self.block_store.clone();
        async move {
            let cid = await!(block_store.put(block))?;
            // sending only fails if no one is listening anymore
            // and that is okay with us.
            let _ = events.send(RepoEvent::ProvideBlock(cid.clone()));
            Ok(cid)
        }
    }

    /// Retrives a block from the block store.
    pub fn get_block(&self, cid: &Cid) ->
    impl Future<Output=Result<Block, Error>>
    {
        let cid = cid.to_owned();
        let events = self.events.clone();
        let block_store = self.block_store.clone();
        async move {
            if !await!(block_store.contains(&cid))? {
                // sending only fails if no one is listening anymore
                // and that is okay with us.
                let _ = events.send(RepoEvent::WantBlock(cid.clone()));
            }
            await!(BlockFuture::new(block_store, cid))
        }
    }

    /// Remove block from the block store.
    pub fn remove_block(&self, cid: &Cid)
        -> impl Future<Output=Result<(), Error>>
    {
        // sending only fails if no one is listening anymore
        // and that is okay with us.
        let _ = self.events.send(RepoEvent::UnprovideBlock(cid.to_owned()));
        self.block_store.remove(cid)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::env::temp_dir;

    #[derive(Clone)]
    pub struct Types;

    impl RepoTypes for Types {
        type TBlockStore = mem::MemBlockStore;
    }

    pub fn create_mock_repo() -> Repo<Types> {
        let mut tmp = temp_dir();
        tmp.push("ipfstools-repo");
        let options: RepoOptions<Types> = RepoOptions {
            _marker: PhantomData,
            path: tmp,
        };
        let (r, _) = Repo::new(options);
        r
    }

    #[test]
    fn test_repo() {
        let mut tmp = temp_dir();
        tmp.push("ipfstools-repo");
        let options: RepoOptions<Types> = RepoOptions {
            _marker: PhantomData,
            path: tmp,
        };
        let (repo, _) = Repo::new(options);
        tokio::run_async(async move {
            await!(repo.init()).unwrap();
        });
    }
}
