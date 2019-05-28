#![feature(box_syntax)]

#[macro_use]
extern crate log;
extern crate rand;
extern crate serde_derive;
extern crate parking_lot;

extern crate chain;
extern crate primitives;
extern crate storage;

pub mod authority;
pub mod beacon_chain;


mod beacon_chain;
mod checkpoint;
mod errors;
pub mod initialise;
pub mod test_utils;
pub use self::beacon_chain::{BeaconChain, BlockProcessingOutcome, InvalidBlock, ValidBlock};
pub use self::checkpoint::CheckPoint;
pub use self::errors::{BeaconChainError, BlockProductionError};
pub use fork_choice;
pub use parking_lot;
pub use slot_clock;
pub use state_processing::per_block_processing::errors::{
    AttestationValidationError, AttesterSlashingValidationError, DepositValidationError,
    ExitValidationError, ProposerSlashingValidationError, TransferValidationError,
};
pub use store;
pub use types;
