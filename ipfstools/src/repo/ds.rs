use crate::block::Cid;

pub use errors::Error;
pub use types::*;
pub use storage::DB;
use std::path::PathBuf;

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

/// An object capable of storing and retrieving objects implementing `StoreItem`.
///
/// A `Store` is fundamentally backed by a key-value database, however it provides support for
/// columns. A simple column implementation might involve prefixing a key with some bytes unique to
/// each column.
pub trait DataStore : Sync + Send + Sized +Clone + Unpin + 'static{

    /// Store an item in `Self`.
    fn put(&self, col: Column, key: &[u8], value: &[u8]) -> Result<(), DBError>;

    /// Retrieve an item from `Self`.
    fn get(&self, col: Column, key: &[u8]) -> Result<Option<Vec<u8>>, Error>;

    /// Returns `true` if the given key represents an item in `Self`.
    fn exists(&self, col: Column, key: &[u8]) -> Result<bool, Error> ;

    /// Remove an item from `Self`.
    fn delete(&self, col: Column, key: &[u8]) -> Result<(), Error> ;

    /// Given the root of an existing block in the store (`start_block_root`), return a parent
    /// block with the specified `slot`.
    ///
    /// Returns `None` if no parent block exists at that slot, or if `slot` is greater than the
    /// slot of `start_block_root`.
    fn get_block_at_preceeding_slot(
        &self,
        start_block_root: Cid,
        slot: Slot,
    ) -> Result<Option<(Cid, BeaconBlock)>, Error> {
        block_at_slot::get_block_at_preceeding_slot(self, slot, start_block_root)
    }
}

/// A on-disk database which implements the DataStore trait.
///
/// This implementation uses RocksDB with default options.
pub struct DBStore {
    path: PathBuf,
    db:Arc<Mutex<Option<DB>>>,
}

impl DBStore {

    fn open(path:&PathBuf) -> Self {
        // Rocks options.
        let mut options = Options::default();
        options.create_if_missing(true);

        // Ensure the path exists.
        fs::create_dir_all(&path).unwrap_or_else(|_| panic!("Unable to create {:?}", &path));
        let db_path = path.join("database");
        let columns = columns.unwrap_or(&COLUMNS);

        if db_path.exists() {
            Self {
                path:db_path,
                db: DB::open_cf(&options, db_path, &COLUMNS)
                    .expect("Unable to open local database"),
            }
        } else {
            let mut db = Self {
                path:db_path,
                db: DB::open(&options, db_path).expect("Unable to open local database"),
            };
            for cf in columns {
                db.create_col(cf).unwrap();
            }
            db
        }
    }

    /// Create a RocksDB column family. Corresponds to the
/// `create_cf()` function on the RocksDB API.
    #[allow(dead_code)]
    fn create_col(&mut self, col: &str) -> Result<(), DBError> {
        match self.db.create_cf(col, &Options::default()) {
            Err(e) => Err(e.into()),
            Ok(_) => Ok(()),
        }
    }
}

impl DataStore for DBStore {

    /// Corresponds to the `get_cf()` method on the RocksDB API.
    /// Will attempt to get the `ColumnFamily` and return an Err
    /// if it fails.
    fn get(&self, col: Column, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {

        let db = self.db.lock().unwrap();
        let db = self.db.as_ref().unwrap();
        let get = self.db.get_cf(cf, &key)?.map(|value| value.to_vec());
        Ok(get)
    }

    /// Set some value for some key on some column.
    ///
    /// Corresponds to the `cf_handle()` method on the RocksDB API.
    /// Will attempt to get the `ColumnFamily` and return an Err
    /// if it fails.
    fn put(&self, col: Column, key: &[u8], value: &[u8]) -> Result<(), DBError> {

        match self.db.cf_handle(col) {
            None => Err(DecodeError::BytesInvalid(
                "Unknown column".to_string(),
            )),
            Some(handle) => self.db.put_cf(handle, key, val).map_err(|e| e.into()),
        }
    }

    /// Returns `true` if the given key represents an item in `Self`.
    fn exists(&self, col: Column, key: &[u8]) -> Result<bool, Error>{
        /*
         * I'm not sure if this is the correct way to read if some
         * block exists. Naively I would expect this to unncessarily
         * copy some data, but I could be wrong.
         */
        match self.db.cf_handle(col) {
            None => Err(DecodeError::BytesInvalid(
                "Unknown column".to_string(),
            )),
            Some(handle) => Ok(self.db.get_cf(handle, key)?.is_some()),
        }
    }

    /// Delete the value for some key on some column.
    ///
    /// Corresponds to the `delete_cf()` method on the RocksDB API.
    /// Will attempt to get the `ColumnFamily` and return an Err
    /// if it fails.
    fn delete(&self, col: Column, key: &[u8]) -> Result<(), Error> {
        match self.db.cf_handle(col) {
            None => Err(DecodeError::BytesInvalid(
                "Unknown column".to_string(),
            )),
            Some(handle) => {
                self.db.delete_cf(handle, key)?;
                Ok(())
            }
        }
    }
}