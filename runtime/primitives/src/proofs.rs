use crate::hash::{hash, hash_struct};

// SinglePartitionProofLen represents the number of bytes in a single partition
// PoRep or PoSt proof. The total length of a PoSt or PoRep proof equals the
// product of SinglePartitionProofLen and the number of partitions.
const SINGLEPARTITIONPROOFLEN: u32 = 192;

// PoStProof is the byte representation of the Proof of SpaceTime proof
pub type PoStProof = hash;

// PoRepProof is the byte representation of the Seal Proof of Replication
pub type PoRepProof = hash;


impl PoStProof {

    // ProofPartitions returns the number of partitions used to create the PoRep
    // proof, or an error if the PoRep proof has an unsupported length.
    //TODO: NOT IMPLEMENT
    pub fn ProofPartitions(){}
}

impl PoRepProof {
    // ProofPartitions returns the number of partitions used to create the PoSt
    // proof, or an error if the PoSt proof has an unsupported length.
    //TODO: NOT IMPLEMENT
    pub fn ProofPartitions(){}
}
