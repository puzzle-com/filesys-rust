//! Repo functionality for Lighthouse.
//!
//! Provides the following stores:
//!
//! - `DataStore`: an on-disk store backed by leveldb. Used in production.
//! - `MemoryStore`: an in-memory store backed by a hash-map. Used for testing.
//!
//! Provides a simple API for storing/retrieving all types that sometimes needs type-hints. See
//! tests for implementation examples.

use crate::error::Error;
use crate::block::Cid;

const API_FILE: &str = "api";
const CONFIG_FILE_NAME: &str = "config.json";
const LOCK_FILE: &str = "repo.lock";
const VERSION_FILENAME: &str = "version";
const WALLET_DATASTORE_FILENAME_PREFIX: &str ="wallet";
const CHAIN_DATASTORE_FILENAME_PREFIX: &str ="chain";
const DEALS_DATASTROE_FILENAME_PREFIX: &str = "deals";
const SNAPSHOT_DATASTORE_FILENAME_PREFIX: &str ="snapshots";

/// Repo is a representation of all persistent data in a FileSys node.
pub trait Repo {

    /// WalletDatastore is a specific storage solution, only used to store sensitive wallet information.
    fn WalletDatastore()-> Resut<(),Error>;

    /// KeystoreDataStore is a specific storage solution, only used to store local keystore information.
    fn KeystoreDataStore() -> Result<(),Error>;

    /// ChainDatastore is a specific storage solution, only used to store already validated chain data.
    fn ChainDatastore() -> Result<(),Error>;

    /// DealsDatastore holds deals data.
    fn DealsDatastore() -> Result<(),Error>;

    /// Version returns the current repo version.
    fn Version() -> Result<(),Error>;

    ///	Path returns the repo path.
    fn Path() -> Result<(),Error>;

}

/// A unique column identifier.
pub enum DBColumn {
    Wallet,
    Keystore,
    BeaconBlock,
    BeaconState,
    BeaconChain,
}

impl<'a> Into<&'a str> for DBColumn {
    /// Returns a `&str` that can be used for keying a key-value data base.
    fn into(self) -> &'a str {
        match self {
            DBColumn::Wallet => &"wat",
            DBColumn::Keystore => &"kst",
            DBColumn::BeaconBlock => &"blk",
            DBColumn::BeaconState => &"ste",
            DBColumn::BeaconChain => &"bch",
        }
    }
}


/// An item that may be stored in a `Store`.
///
/// Provides default methods that are suitable for most applications, however when overridden they
/// provide full customizability of `Store` operations.
pub trait StoreItem : Sized {

    /// Identifies which column this item should be placed in.
    fn db_column() -> DBColumn;

    /// Serialize `self` as bytes.
    fn as_store_bytes(&self) -> Vec<u8>;

    /// De-serialize `self` from bytes.
    fn from_store_bytes(bytes: &mut [u8]) -> Result<Self, Error>;

    /// Store `self`.
    fn db_put(&self, store: &impl Store, key: &Cid) -> Result<(), Error> {
        let column = Self::db_column().into();
        let key = key.as_bytes();

        store
            .put_bytes(column, key, &self.as_store_bytes())
            .map_err(Into::into)
    }

    /// Retrieve an instance of `Self`.
    fn db_get(store: &impl Store, key: &Cid) -> Result<Option<Self>, Error> {
        let column = Self::db_column().into();
        let key = key.as_bytes();

        match store.get_bytes(column, key)? {
            Some(mut bytes) => Ok(Some(Self::from_store_bytes(&mut bytes[..])?)),
            None => Ok(None),
        }
    }

    /// Return `true` if an instance of `Self` exists in `Store`.
    fn db_exists(store: &impl Store, key: &Cid) -> Result<bool, Error> {
        let column = Self::db_column().into();
        let key = key.as_bytes();

        store.key_exists(column, key)
    }

    /// Delete `self` from the `Store`.
    fn db_delete(store: &impl Store, key: &Cid) -> Result<(), Error> {
        let column = Self::db_column().into();
        let key = key.as_bytes();

        store.key_delete(column, key)
    }

}

/// An object capable of storing and retrieving objects implementing `StoreItem`.
///
/// A `Store` is fundamentally backed by a key-value database, however it provides support for
/// columns. A simple column implementation might involve prefixing a key with some bytes unique to
/// each column.
pub trait DataStore : Sync + Send + Sized {
    /// Store an item in `Self`.
    fn put(&self, key: &Hash256, item: &impl StoreItem) -> Result<(), Error> {
        item.db_put(self, key)
    }

    /// Retrieve an item from `Self`.
    fn get<I: StoreItem>(&self, key: &Cid) -> Result<Option<I>, Error> {
        I::db_get(self, key)
    }

    /// Returns `true` if the given key represents an item in `Self`.
    fn exists<I: StoreItem>(&self, key: &Cid) -> Result<bool, Error> {
        I::db_exists(self, key)
    }

    /// Remove an item from `Self`.
    fn delete<I: StoreItem>(&self, key: &Cid) -> Result<(), Error> {
        I::db_delete(self, key)
    }

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

    /// Retrieve some bytes in `column` with `key`.
    fn get_bytes(&self, column: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Error>;

    /// Store some `value` in `column`, indexed with `key`.
    fn put_bytes(&self, column: &str, key: &[u8], value: &[u8]) -> Result<(), Error>;

    /// Return `true` if `key` exists in `column`.
    fn key_exists(&self, column: &str, key: &[u8]) -> Result<bool, Error>;

    /// Removes `key` from `column`.
    fn key_delete(&self, column: &str, key: &[u8]) -> Result<(), Error>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
