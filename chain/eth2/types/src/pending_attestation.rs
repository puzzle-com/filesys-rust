use crate::test_utils::TestRandom;
use crate::{Attestation, AttestationData, Bitfield, Slot};

use serde_derive::{Deserialize, Serialize};
use ssz_derive::{Decode, Encode};
use test_random_derive::TestRandom;
use tree_hash_derive::{CachedTreeHash, TreeHash};

/// An attestation that has been included in the state but not yet fully processed.
///
/// Spec v0.5.1
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    TreeHash,
    CachedTreeHash,
    TestRandom,
)]
pub struct PendingAttestation {
    pub aggregation_bitfield: Bitfield,
    pub data: AttestationData,
    pub custody_bitfield: Bitfield,
    pub inclusion_slot: Slot,
}

impl PendingAttestation {
    /// Create a `PendingAttestation` from an `Attestation`, at the given `inclusion_slot`.
    pub fn from_attestation(attestation: &Attestation, inclusion_slot: Slot) -> Self {
        PendingAttestation {
            data: attestation.data.clone(),
            aggregation_bitfield: attestation.aggregation_bitfield.clone(),
            custody_bitfield: attestation.custody_bitfield.clone(),
            inclusion_slot,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    ssz_tests!(PendingAttestation);
    cached_tree_hash_tests!(PendingAttestation);
}
