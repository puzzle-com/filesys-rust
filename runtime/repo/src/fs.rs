use crate::block::Cid;
use StoreItem;

#[derive(Clone, Debug)]
pub struct FsRepo {
    root: PathBuf,
    cids: Arc<Mutex<HashSet<Cid>>>,
    keystore:StoreItem,
    chain:StoreItem,
    wallet:StoreItem,
}
