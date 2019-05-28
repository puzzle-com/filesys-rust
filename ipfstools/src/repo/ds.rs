use crate::block::Cid;

pub use errors::Error;
pub use types::*;

/// An object capable of storing and retrieving objects implementing `StoreItem`.
///
/// A `Store` is fundamentally backed by a key-value database, however it provides support for
/// columns. A simple column implementation might involve prefixing a key with some bytes unique to
/// each column.
pub trait DataStore : Sync + Send + Sized {

    /// Store an item in `Self`.
    fn put(&self, cid: &Cid, item: &impl StoreItem) -> Result<(), Error> {
        item.db_put(self, cid)
    }

    /// Retrieve an item from `Self`.
    fn get<I: StoreItem>(&self, cid: &Cid) -> Result<Option<I>, Error> {
        I::db_get(self, cid)
    }

    /// Returns `true` if the given key represents an item in `Self`.
    fn exists<I: StoreItem>(&self, cid: &Cid) -> Result<bool, Error> {
        I::db_exists(self, cid)
    }

    /// Remove an item from `Self`.
    fn delete<I: StoreItem>(&self, cid: &Cid) -> Result<(), Error> {
        I::db_delete(self, cid)
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

    /// Retrieve some cid in `column` with `key`.
    fn get_bytes(&self, column: &str, key: &Cid) -> Result<Option<Vec<Cid>>, Error>;

    /// Store some `value` in `column`, indexed with `key`.
    fn put_bytes(&self, column: &str, key: &Cid, value: &[u8]) -> Result<(), Error>;

    /// Return `true` if `key` exists in `column`.
    fn key_exists(&self, column: &str, key: &Cid) -> Result<bool, Error>;

    /// Removes `key` from `column`.
    fn key_delete(&self, column: &str, key: &Cid) -> Result<(), Error>;

}

pub trait StoreItem : Sized {
    /// Identifies which column this item should be placed in.
    fn db_column() -> DBColumn;

    /// Serialize `self` as bytes.
    fn as_store_bytes(&self) -> Vec<Cid>;

    /// De-serialize `self` from bytes.
    fn from_store_bytes(bytes: &mut [Cid]) -> Result<Self, Error>;

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