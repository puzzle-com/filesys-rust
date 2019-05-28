use crate::checkpoint::CheckPoint;
use crate::errors::{BeaconChainError as Error, BlockProductionError};
use fork_choice::{ForkChoice, ForkChoiceError};
use log::{debug, trace};
use operation_pool::DepositInsertStatus;
use operation_pool::OperationPool;
use parking_lot::{RwLock, RwLockReadGuard};
use slot_clock::SlotClock;
use state_processing::per_block_processing::errors::{
    AttestationValidationError, AttesterSlashingValidationError, DepositValidationError,
    ExitValidationError, ProposerSlashingValidationError, TransferValidationError,
};
use state_processing::{
    per_block_processing, per_block_processing_without_verifying_block_signature,
    per_slot_processing, BlockProcessingError, SlotProcessingError,
};
use std::sync::Arc;
use store::{Error as DBError, Store};
use types::*;


use std::sync::{Arc, RwLock};
use configs::authority::get_authority_config;
use node_runtime::chain_spec::ChainSpec;
use primitives::beacon::{SignedBeaconBlock, SignedBeaconBlockHeader};
use storage::BeaconChainStorage;
use crate::authority::{get_authority, Authority};

pub type BeaconBlockChain =
    chain::BlockChain<SignedBeaconBlockHeader, SignedBeaconBlock, BeaconChainStorage>;

pub struct BeaconClient {
    pub chain: BeaconBlockChain,
    pub authority: RwLock<Box<Authority>>,
}

impl BeaconClient {
    pub fn new(
        genesis: SignedBeaconBlock,
        chain_spec: &ChainSpec,
        storage: Arc<RwLock<BeaconChainStorage>>,
    ) -> Self {
        let chain = chain::BlockChain::new(genesis, storage.clone());
        let authority_config = get_authority_config(chain_spec);
        let authority = RwLock::new(get_authority(authority_config, &chain, storage));
        BeaconClient { chain, authority }
    }
}

#[cfg(test)]
mod tests {
    use chain::test_utils::get_blockchain_storage;
    use chain::BlockChain;
    use node_runtime::chain_spec::{AuthorityRotation, DefaultIdType};
    use primitives::block_traits::SignedBlock;
    use primitives::block_traits::SignedHeader;
    use primitives::crypto::signer::InMemorySigner;
    use primitives::hash::CryptoHash;
    use primitives::types::BlockId;
    use storage::test_utils::create_beacon_shard_storages;

    use super::*;

    fn get_beacon_client() -> (BeaconClient, SignedBeaconBlock) {
        let storage = create_beacon_shard_storages().0;
        let genesis =
            SignedBeaconBlock::new(0, CryptoHash::default(), vec![], CryptoHash::default());
        let chain_spec = ChainSpec::testing_spec(
            DefaultIdType::Named,
            3,
            1,
            AuthorityRotation::ThresholdedProofOfStake { epoch_length: 2, num_seats_per_slot: 1 },
        )
        .0;
        let beacon_client = BeaconClient::new(genesis.clone(), &chain_spec, storage);
        (beacon_client, genesis)
    }

    #[test]
    fn test_genesis() {
        let (bc, genesis) = get_beacon_client();
        assert_eq!(bc.chain.get_block(&BlockId::Hash(genesis.block_hash())).unwrap(), genesis);
        assert_eq!(bc.chain.get_block(&BlockId::Number(0)).unwrap(), genesis);
    }

    #[test]
    #[should_panic]
    fn test_invalid_genesis() {
        let (bc, _) = get_beacon_client();
        let storage = get_blockchain_storage(bc.chain);
        let invalid_genesis_block =
            SignedBeaconBlock::new(1, CryptoHash::default(), vec![], CryptoHash::default());
        let _ = BlockChain::new(invalid_genesis_block, storage);
    }

    #[test]
    fn test_restart_chain() {
        let (bc, genesis) = get_beacon_client();
        let mut block1 =
            SignedBeaconBlock::new(1, genesis.block_hash(), vec![], CryptoHash::default());
        let signer = Arc::new(InMemorySigner::from_random());
        let sig = block1.sign(&*signer);
        block1.add_signature(&sig, 0);
        bc.chain.insert_block(block1.clone());
        let best_block_header = bc.chain.best_header();
        assert_eq!(best_block_header.block_hash(), block1.block_hash());
        assert_eq!(best_block_header.index(), 1);
        // Create new BlockChain that reads from the same storage.
        let storage = get_blockchain_storage(bc.chain);
        let other_bc = BlockChain::new(genesis.clone(), storage);
        assert_eq!(other_bc.best_hash(), block1.block_hash());
        assert_eq!(other_bc.best_index(), 1);
        assert_eq!(other_bc.get_block(&BlockId::Hash(block1.block_hash())).unwrap(), block1);
    }

    #[test]
    fn test_light_client() {
        let (bc, genesis) = get_beacon_client();
        let block1 = SignedBeaconBlock::new(1, genesis.block_hash(), vec![], CryptoHash::default());
        let block1_hash = block1.block_hash();
        bc.chain.insert_header(block1.header());
        assert_eq!(bc.chain.best_index(), 1);
        assert!(bc.chain.is_known_header(&block1_hash));
        assert!(!bc.chain.is_known_block(&block1_hash));
        let block2 = SignedBeaconBlock::new(2, block1_hash, vec![], CryptoHash::default());
        bc.chain.insert_header(block2.header());
        assert_eq!(bc.chain.best_index(), 2);
        bc.chain.insert_block(block1);
        assert_eq!(bc.chain.best_index(), 2);
        assert!(bc.chain.is_known_block(&block1_hash));
    }

    //    fn test_fork_choice_rule_helper(graph: Vec<(u32, u32, usize)>, expect: u32) {
    //        let storage = Arc::new(create_memory_db());
    //
    //        let genesis =
    //            SignedBeaconBlock::new(0, CryptoHash::default(), vec![], CryptoHash::default());
    //        let bc = BlockChain::new(genesis.clone(), storage);
    //        let mut blocks: HashMap<u32, SignedBeaconBlock> = HashMap::new();
    //        blocks.insert(0, genesis.clone());
    //
    //        for (self_id, parent_id, sign_count) in graph.iter() {
    //            let mut block;
    //            {
    //                let parent = blocks.get(parent_id).unwrap();
    //                block = SignedBeaconBlock::new(
    //                    parent.body.header.index + 1,
    //                    parent.block_hash(),
    //                    vec![],
    //                    hash(&[*self_id as u8]),
    //                );
    //            }
    //            for i in 0..*sign_count {
    //                // Having proper signing here is far too slow, and unnecessary for this test
    //                let sig = BlsSignature::empty();
    //                block.add_signature(&sig, i);
    //            }
    //            blocks.insert(*self_id, block.clone());
    //            assert_eq!(bc.insert_block(block.clone()), false);
    //        }
    //        let best_hash = bc.best_block().block_hash();
    //        assert_eq!(best_hash, blocks.get(&expect).unwrap().block_hash());
    //    }
    //
    //    #[test]
    //    fn test_fork_choice_rule() {
    //        // First 3 examples from https://ethresear.ch/t/immediate-message-driven-ghost-as-ffg-fork-choice-rule/2561
    //
    //        //    15 - 16 - 65
    //        //  /
    //        // -
    //        //  \
    //        //    55 - 56
    //        //
    //        // We prefer the bottom fork, even though the top is longer.
    //        test_fork_choice_rule_helper(
    //            vec![(1, 0, 15), (2, 1, 16), (3, 2, 65), (4, 0, 55), (5, 4, 56)],
    //            5,
    //        );
    //        test_fork_choice_rule_helper(
    //            vec![(4, 0, 55), (5, 4, 56), (1, 0, 15), (2, 1, 16), (3, 2, 65)],
    //            5,
    //        );
    //
    //        //    15 - 51
    //        //  /
    //        // -
    //        //  \
    //        //    65 - 20
    //        test_fork_choice_rule_helper(vec![(1, 0, 15), (2, 0, 65), (3, 1, 51), (4, 2, 20)], 4);
    //
    //        //    40 - 51
    //        //  /
    //        // -
    //        //  \
    //        //    60 - 5
    //        test_fork_choice_rule_helper(vec![(1, 0, 40), (2, 0, 60), (3, 1, 51), (4, 2, 5)], 3);
    //
    //        //    65 - 20
    //        //  /
    //        // -
    //        //  \      35
    //        //   \   /
    //        //     30
    //        //       \
    //        //         40
    //        //
    //        // If we were using GHOST, we would prefer the bottom fork, because at each step the total
    //        // subtree weight of the lower fork is higher.  As is, we prefer the top fork.
    //        test_fork_choice_rule_helper(
    //            vec![(1, 0, 65), (2, 1, 20), (3, 0, 30), (4, 3, 35), (5, 3, 40)],
    //            2,
    //        );
    //    }
}



#[derive(Debug, PartialEq)]
pub enum ValidBlock {
    /// The block was successfully processed.
    Processed,
}

#[derive(Debug, PartialEq)]
pub enum InvalidBlock {
    /// The block slot is greater than the present slot.
    FutureSlot {
        present_slot: Slot,
        block_slot: Slot,
    },
    /// The block state_root does not match the generated state.
    StateRootMismatch,
    /// The blocks parent_root is unknown.
    ParentUnknown,
    /// There was an error whilst advancing the parent state to the present slot. This condition
    /// should not occur, it likely represents an internal error.
    SlotProcessingError(SlotProcessingError),
    /// The block could not be applied to the state, it is invalid.
    PerBlockProcessingError(BlockProcessingError),
}

#[derive(Debug, PartialEq)]
pub enum BlockProcessingOutcome {
    /// The block was successfully validated.
    ValidBlock(ValidBlock),
    /// The block was not successfully validated.
    InvalidBlock(InvalidBlock),
}

impl BlockProcessingOutcome {
    /// Returns `true` if the block was objectively invalid and we should disregard the peer who
    /// sent it.
    pub fn is_invalid(&self) -> bool {
        match self {
            BlockProcessingOutcome::ValidBlock(_) => false,
            BlockProcessingOutcome::InvalidBlock(r) => match r {
                InvalidBlock::FutureSlot { .. } => true,
                InvalidBlock::StateRootMismatch => true,
                InvalidBlock::ParentUnknown => false,
                InvalidBlock::SlotProcessingError(_) => false,
                InvalidBlock::PerBlockProcessingError(e) => match e {
                    BlockProcessingError::Invalid(_) => true,
                    BlockProcessingError::BeaconStateError(_) => false,
                },
            },
        }
    }

    /// Returns `true` if the block was successfully processed and can be removed from any import
    /// queues or temporary storage.
    pub fn sucessfully_processed(&self) -> bool {
        match self {
            BlockProcessingOutcome::ValidBlock(_) => true,
            _ => false,
        }
    }
}

pub struct BeaconChain<T, U, F, E: EthSpec> {
    pub store: Arc<T>,
    pub slot_clock: U,
    pub op_pool: OperationPool<E>,
    canonical_head: RwLock<CheckPoint<E>>,
    finalized_head: RwLock<CheckPoint<E>>,
    pub state: RwLock<BeaconState<E>>,
    pub spec: ChainSpec,
    pub fork_choice: RwLock<F>,
}

impl<T, U, F, E> BeaconChain<T, U, F, E>
    where
        T: Store,
        U: SlotClock,
        F: ForkChoice,
        E: EthSpec,
{
    /// Instantiate a new Beacon Chain, from genesis.
    pub fn from_genesis(
        store: Arc<T>,
        slot_clock: U,
        mut genesis_state: BeaconState<E>,
        genesis_block: BeaconBlock,
        spec: ChainSpec,
        fork_choice: F,
    ) -> Result<Self, Error> {
        let state_root = genesis_state.canonical_root();
        store.put(&state_root, &genesis_state)?;

        let block_root = genesis_block.block_header().canonical_root();
        store.put(&block_root, &genesis_block)?;

        let finalized_head = RwLock::new(CheckPoint::new(
            genesis_block.clone(),
            block_root,
            genesis_state.clone(),
            state_root,
        ));
        let canonical_head = RwLock::new(CheckPoint::new(
            genesis_block.clone(),
            block_root,
            genesis_state.clone(),
            state_root,
        ));

        genesis_state.build_all_caches(&spec)?;

        Ok(Self {
            store,
            slot_clock,
            op_pool: OperationPool::new(),
            state: RwLock::new(genesis_state),
            finalized_head,
            canonical_head,
            spec,
            fork_choice: RwLock::new(fork_choice),
        })
    }

    /// Returns the beacon block body for each beacon block root in `roots`.
    ///
    /// Fails if any root in `roots` does not have a corresponding block.
    pub fn get_block_bodies(&self, roots: &[Hash256]) -> Result<Vec<BeaconBlockBody>, Error> {
        let bodies: Result<Vec<BeaconBlockBody>, _> = roots
            .iter()
            .map(|root| match self.get_block(root)? {
                Some(block) => Ok(block.body),
                None => Err(Error::DBInconsistent("Missing block".into())),
            })
            .collect();

        Ok(bodies?)
    }

    /// Returns the beacon block header for each beacon block root in `roots`.
    ///
    /// Fails if any root in `roots` does not have a corresponding block.
    pub fn get_block_headers(&self, roots: &[Hash256]) -> Result<Vec<BeaconBlockHeader>, Error> {
        let headers: Result<Vec<BeaconBlockHeader>, _> = roots
            .iter()
            .map(|root| match self.get_block(root)? {
                Some(block) => Ok(block.block_header()),
                None => Err(Error::DBInconsistent("Missing block".into())),
            })
            .collect();

        Ok(headers?)
    }

    /// Returns `count `beacon block roots, starting from `start_slot` with an
    /// interval of `skip` slots between each root.
    ///
    /// ## Errors:
    ///
    /// - `SlotOutOfBounds`: Unable to return the full specified range.
    /// - `SlotOutOfBounds`: Unable to load a state from the DB.
    /// - `SlotOutOfBounds`: Start slot is higher than the first slot.
    /// - Other: BeaconState` is inconsistent.
    pub fn get_block_roots(
        &self,
        earliest_slot: Slot,
        count: usize,
        skip: usize,
    ) -> Result<Vec<Hash256>, Error> {
        let step_by = Slot::from(skip + 1);

        let mut roots: Vec<Hash256> = vec![];

        // The state for reading block roots. Will be updated with an older state if slots go too
        // far back in history.
        let mut state = self.state.read().clone();

        // The final slot in this series, will be reduced by `skip` each loop iteration.
        let mut slot = earliest_slot + Slot::from(count * (skip + 1)) - 1;

        // If the highest slot requested is that of the current state insert the root of the
        // head block, unless the head block's slot is not matching.
        if slot == state.slot && self.head().beacon_block.slot == slot {
            roots.push(self.head().beacon_block_root);

            slot -= step_by;
        } else if slot >= state.slot {
            return Err(BeaconStateError::SlotOutOfBounds.into());
        }

        loop {
            // If the slot is within the range of the current state's block roots, append the root
            // to the output vec.
            //
            // If we get `SlotOutOfBounds` error, load the oldest available historic
            // state from the DB.
            match state.get_block_root(slot) {
                Ok(root) => {
                    if slot < earliest_slot {
                        break;
                    } else {
                        roots.push(*root);
                        slot -= step_by;
                    }
                }
                Err(BeaconStateError::SlotOutOfBounds) => {
                    // Read the earliest historic state in the current slot.
                    let earliest_historic_slot =
                        state.slot - Slot::from(E::SlotsPerHistoricalRoot::to_usize());
                    // Load the earlier state from disk.
                    let new_state_root = state.get_state_root(earliest_historic_slot)?;

                    // Break if the DB is unable to load the state.
                    state = match self.store.get(&new_state_root) {
                        Ok(Some(state)) => state,
                        _ => break,
                    }
                }
                Err(e) => return Err(e.into()),
            };
        }

        // Return the results if they pass a sanity check.
        if (slot <= earliest_slot) && (roots.len() == count) {
            // Reverse the ordering of the roots. We extracted them in reverse order to make it
            // simpler to lookup historic states.
            //
            // This is a potential optimisation target.
            Ok(roots.iter().rev().cloned().collect())
        } else {
            Err(BeaconStateError::SlotOutOfBounds.into())
        }
    }

    /// Returns the block at the given root, if any.
    ///
    /// ## Errors
    ///
    /// May return a database error.
    pub fn get_block(&self, block_root: &Hash256) -> Result<Option<BeaconBlock>, Error> {
        Ok(self.store.get(block_root)?)
    }

    /// Update the canonical head to some new values.
    pub fn update_canonical_head(
        &self,
        new_beacon_block: BeaconBlock,
        new_beacon_block_root: Hash256,
        new_beacon_state: BeaconState<E>,
        new_beacon_state_root: Hash256,
    ) {
        debug!(
            "Updating canonical head with block at slot: {}",
            new_beacon_block.slot
        );
        let mut head = self.canonical_head.write();
        head.update(
            new_beacon_block,
            new_beacon_block_root,
            new_beacon_state,
            new_beacon_state_root,
        );
    }

    /// Returns a read-lock guarded `CheckPoint` struct for reading the head (as chosen by the
    /// fork-choice rule).
    ///
    /// It is important to note that the `beacon_state` returned may not match the present slot. It
    /// is the state as it was when the head block was received, which could be some slots prior to
    /// now.
    pub fn head(&self) -> RwLockReadGuard<CheckPoint<E>> {
        self.canonical_head.read()
    }

    /// Updates the canonical `BeaconState` with the supplied state.
    ///
    /// Advances the chain forward to the present slot. This method is better than just setting
    /// state and calling `catchup_state` as it will not result in an old state being installed and
    /// then having it iteratively updated -- in such a case it's possible for another thread to
    /// find the state at an old slot.
    pub fn update_state(&self, mut state: BeaconState<E>) -> Result<(), Error> {
        let present_slot = match self.slot_clock.present_slot() {
            Ok(Some(slot)) => slot,
            _ => return Err(Error::UnableToReadSlot),
        };

        // If required, transition the new state to the present slot.
        for _ in state.slot.as_u64()..present_slot.as_u64() {
            per_slot_processing(&mut state, &self.spec)?;
        }

        state.build_all_caches(&self.spec)?;

        *self.state.write() = state;

        Ok(())
    }

    /// Ensures the current canonical `BeaconState` has been transitioned to match the `slot_clock`.
    pub fn catchup_state(&self) -> Result<(), Error> {
        let present_slot = match self.slot_clock.present_slot() {
            Ok(Some(slot)) => slot,
            _ => return Err(Error::UnableToReadSlot),
        };

        let mut state = self.state.write();

        // If required, transition the new state to the present slot.
        for _ in state.slot.as_u64()..present_slot.as_u64() {
            // Ensure the next epoch state caches are built in case of an epoch transition.
            state.build_epoch_cache(RelativeEpoch::NextWithoutRegistryChange, &self.spec)?;
            state.build_epoch_cache(RelativeEpoch::NextWithRegistryChange, &self.spec)?;

            per_slot_processing(&mut *state, &self.spec)?;
        }

        state.build_all_caches(&self.spec)?;

        Ok(())
    }

    /// Build all of the caches on the current state.
    ///
    /// Ideally this shouldn't be required, however we leave it here for testing.
    pub fn ensure_state_caches_are_built(&self) -> Result<(), Error> {
        self.state.write().build_all_caches(&self.spec)?;

        Ok(())
    }

    /// Update the justified head to some new values.
    pub fn update_finalized_head(
        &self,
        new_beacon_block: BeaconBlock,
        new_beacon_block_root: Hash256,
        new_beacon_state: BeaconState<E>,
        new_beacon_state_root: Hash256,
    ) {
        let mut finalized_head = self.finalized_head.write();
        finalized_head.update(
            new_beacon_block,
            new_beacon_block_root,
            new_beacon_state,
            new_beacon_state_root,
        );
    }

    /// Returns a read-lock guarded `CheckPoint` struct for reading the justified head (as chosen,
    /// indirectly,  by the fork-choice rule).
    pub fn finalized_head(&self) -> RwLockReadGuard<CheckPoint<E>> {
        self.finalized_head.read()
    }

    /// Returns the validator index (if any) for the given public key.
    ///
    /// Information is retrieved from the present `beacon_state.validator_registry`.
    pub fn validator_index(&self, pubkey: &PublicKey) -> Option<usize> {
        for (i, validator) in self
            .head()
            .beacon_state
            .validator_registry
            .iter()
            .enumerate()
            {
                if validator.pubkey == *pubkey {
                    return Some(i);
                }
            }
        None
    }

    /// Reads the slot clock, returns `None` if the slot is unavailable.
    ///
    /// The slot might be unavailable due to an error with the system clock, or if the present time
    /// is before genesis (i.e., a negative slot).
    ///
    /// This is distinct to `present_slot`, which simply reads the latest state. If a
    /// call to `read_slot_clock` results in a higher slot than a call to `present_slot`,
    /// `self.state` should undergo per slot processing.
    pub fn read_slot_clock(&self) -> Option<Slot> {
        match self.slot_clock.present_slot() {
            Ok(Some(some_slot)) => Some(some_slot),
            Ok(None) => None,
            _ => None,
        }
    }

    /// Reads the slot clock (see `self.read_slot_clock()` and returns the number of slots since
    /// genesis.
    pub fn slots_since_genesis(&self) -> Option<SlotHeight> {
        let now = self.read_slot_clock()?;

        if now < self.spec.genesis_slot {
            None
        } else {
            Some(SlotHeight::from(
                now.as_u64() - self.spec.genesis_slot.as_u64(),
            ))
        }
    }

    /// Returns slot of the present state.
    ///
    /// This is distinct to `read_slot_clock`, which reads from the actual system clock. If
    /// `self.state` has not been transitioned it is possible for the system clock to be on a
    /// different slot to what is returned from this call.
    pub fn present_slot(&self) -> Slot {
        self.state.read().slot
    }

    /// Returns the block proposer for a given slot.
    ///
    /// Information is read from the present `beacon_state` shuffling, so only information from the
    /// present and prior epoch is available.
    pub fn block_proposer(&self, slot: Slot) -> Result<usize, BeaconStateError> {
        self.state
            .write()
            .build_epoch_cache(RelativeEpoch::Current, &self.spec)?;

        let index = self.state.read().get_beacon_proposer_index(
            slot,
            RelativeEpoch::Current,
            &self.spec,
        )?;

        Ok(index)
    }

    /// Returns the attestation slot and shard for a given validator index.
    ///
    /// Information is read from the current state, so only information from the present and prior
    /// epoch is available.
    pub fn validator_attestion_slot_and_shard(
        &self,
        validator_index: usize,
    ) -> Result<Option<(Slot, u64)>, BeaconStateError> {
        trace!(
            "BeaconChain::validator_attestion_slot_and_shard: validator_index: {}",
            validator_index
        );
        if let Some(attestation_duty) = self
            .state
            .read()
            .get_attestation_duties(validator_index, &self.spec)?
        {
            Ok(Some((attestation_duty.slot, attestation_duty.shard)))
        } else {
            Ok(None)
        }
    }

    /// Produce an `AttestationData` that is valid for the present `slot` and given `shard`.
    pub fn produce_attestation_data(&self, shard: u64) -> Result<AttestationData, Error> {
        trace!("BeaconChain::produce_attestation: shard: {}", shard);
        let state = self.state.read();

        let current_epoch_start_slot = self
            .state
            .read()
            .slot
            .epoch(self.spec.slots_per_epoch)
            .start_slot(self.spec.slots_per_epoch);

        let target_root = if state.slot == current_epoch_start_slot {
            // If we're on the first slot of the state's epoch.
            if self.head().beacon_block.slot == state.slot {
                // If the current head block is from the current slot, use its block root.
                self.head().beacon_block_root
            } else {
                // If the current head block is not from this slot, use the slot from the previous
                // epoch.
                *self
                    .state
                    .read()
                    .get_block_root(current_epoch_start_slot - self.spec.slots_per_epoch)?
            }
        } else {
            // If we're not on the first slot of the epoch.
            *self.state.read().get_block_root(current_epoch_start_slot)?
        };

        Ok(AttestationData {
            slot: self.state.read().slot,
            shard,
            beacon_block_root: self.head().beacon_block_root,
            target_root,
            crosslink_data_root: Hash256::zero(),
            previous_crosslink: state.latest_crosslinks[shard as usize].clone(),
            source_epoch: state.current_justified_epoch,
            source_root: state.current_justified_root,
        })
    }

    /// Accept a new attestation from the network.
    ///
    /// If valid, the attestation is added to the `op_pool` and aggregated with another attestation
    /// if possible.
    pub fn process_attestation(
        &self,
        attestation: Attestation,
    ) -> Result<(), AttestationValidationError> {
        self.op_pool
            .insert_attestation(attestation, &*self.state.read(), &self.spec)
    }

    /// Accept some deposit and queue it for inclusion in an appropriate block.
    pub fn process_deposit(
        &self,
        deposit: Deposit,
    ) -> Result<DepositInsertStatus, DepositValidationError> {
        self.op_pool
            .insert_deposit(deposit, &*self.state.read(), &self.spec)
    }

    /// Accept some exit and queue it for inclusion in an appropriate block.
    pub fn process_voluntary_exit(&self, exit: VoluntaryExit) -> Result<(), ExitValidationError> {
        self.op_pool
            .insert_voluntary_exit(exit, &*self.state.read(), &self.spec)
    }

    /// Accept some transfer and queue it for inclusion in an appropriate block.
    pub fn process_transfer(&self, transfer: Transfer) -> Result<(), TransferValidationError> {
        self.op_pool
            .insert_transfer(transfer, &*self.state.read(), &self.spec)
    }

    /// Accept some proposer slashing and queue it for inclusion in an appropriate block.
    pub fn process_proposer_slashing(
        &self,
        proposer_slashing: ProposerSlashing,
    ) -> Result<(), ProposerSlashingValidationError> {
        self.op_pool
            .insert_proposer_slashing(proposer_slashing, &*self.state.read(), &self.spec)
    }

    /// Accept some attester slashing and queue it for inclusion in an appropriate block.
    pub fn process_attester_slashing(
        &self,
        attester_slashing: AttesterSlashing,
    ) -> Result<(), AttesterSlashingValidationError> {
        self.op_pool
            .insert_attester_slashing(attester_slashing, &*self.state.read(), &self.spec)
    }

    /// Accept some block and attempt to add it to block DAG.
    ///
    /// Will accept blocks from prior slots, however it will reject any block from a future slot.
    pub fn process_block(&self, block: BeaconBlock) -> Result<BlockProcessingOutcome, Error> {
        debug!("Processing block with slot {}...", block.slot);

        let block_root = block.block_header().canonical_root();

        let present_slot = self.present_slot();

        if block.slot > present_slot {
            return Ok(BlockProcessingOutcome::InvalidBlock(
                InvalidBlock::FutureSlot {
                    present_slot,
                    block_slot: block.slot,
                },
            ));
        }

        // Load the blocks parent block from the database, returning invalid if that block is not
        // found.
        let parent_block_root = block.previous_block_root;
        let parent_block: BeaconBlock = match self.store.get(&parent_block_root)? {
            Some(previous_block_root) => previous_block_root,
            None => {
                return Ok(BlockProcessingOutcome::InvalidBlock(
                    InvalidBlock::ParentUnknown,
                ));
            }
        };

        // Load the parent blocks state from the database, returning an error if it is not found.
        // It is an error because if know the parent block we should also know the parent state.
        let parent_state_root = parent_block.state_root;
        let parent_state = self
            .store
            .get(&parent_state_root)?
            .ok_or_else(|| Error::DBInconsistent(format!("Missing state {}", parent_state_root)))?;

        // TODO: check the block proposer signature BEFORE doing a state transition. This will
        // significantly lower exposure surface to DoS attacks.

        // Transition the parent state to the block slot.
        let mut state: BeaconState<E> = parent_state;
        for _ in state.slot.as_u64()..block.slot.as_u64() {
            if let Err(e) = per_slot_processing(&mut state, &self.spec) {
                return Ok(BlockProcessingOutcome::InvalidBlock(
                    InvalidBlock::SlotProcessingError(e),
                ));
            }
        }

        // Apply the received block to its parent state (which has been transitioned into this
        // slot).
        if let Err(e) = per_block_processing(&mut state, &block, &self.spec) {
            return Ok(BlockProcessingOutcome::InvalidBlock(
                InvalidBlock::PerBlockProcessingError(e),
            ));
        }

        let state_root = state.canonical_root();

        if block.state_root != state_root {
            return Ok(BlockProcessingOutcome::InvalidBlock(
                InvalidBlock::StateRootMismatch,
            ));
        }

        // Store the block and state.
        self.store.put(&block_root, &block)?;
        self.store.put(&state_root, &state)?;

        // run the fork_choice add_block logic
        self.fork_choice
            .write()
            .add_block(&block, &block_root, &self.spec)?;

        // If the parent block was the parent_block, automatically update the canonical head.
        //
        // TODO: this is a first-in-best-dressed scenario that is not ideal; fork_choice should be
        // run instead.
        if self.head().beacon_block_root == parent_block_root {
            self.update_canonical_head(block.clone(), block_root, state.clone(), state_root);

            // Update the canonical `BeaconState`.
            self.update_state(state)?;
        }

        Ok(BlockProcessingOutcome::ValidBlock(ValidBlock::Processed))
    }

    /// Produce a new block at the present slot.
    ///
    /// The produced block will not be inherently valid, it must be signed by a block producer.
    /// Block signing is out of the scope of this function and should be done by a separate program.
    pub fn produce_block(
        &self,
        randao_reveal: Signature,
    ) -> Result<(BeaconBlock, BeaconState<E>), BlockProductionError> {
        debug!("Producing block at slot {}...", self.state.read().slot);

        let mut state = self.state.read().clone();

        state.build_epoch_cache(RelativeEpoch::Current, &self.spec)?;

        trace!("Finding attestations for new block...");

        let previous_block_root = *state
            .get_block_root(state.slot - 1)
            .map_err(|_| BlockProductionError::UnableToGetBlockRootFromState)?;

        let (proposer_slashings, attester_slashings) =
            self.op_pool.get_slashings(&*self.state.read(), &self.spec);

        let mut block = BeaconBlock {
            slot: state.slot,
            previous_block_root,
            state_root: Hash256::zero(), // Updated after the state is calculated.
            signature: self.spec.empty_signature.clone(), // To be completed by a validator.
            body: BeaconBlockBody {
                randao_reveal,
                eth1_data: Eth1Data {
                    // TODO: replace with real data
                    deposit_root: Hash256::zero(),
                    block_hash: Hash256::zero(),
                },
                proposer_slashings,
                attester_slashings,
                attestations: self
                    .op_pool
                    .get_attestations(&*self.state.read(), &self.spec),
                deposits: self.op_pool.get_deposits(&*self.state.read(), &self.spec),
                voluntary_exits: self
                    .op_pool
                    .get_voluntary_exits(&*self.state.read(), &self.spec),
                transfers: self.op_pool.get_transfers(&*self.state.read(), &self.spec),
            },
        };

        debug!(
            "Produced block with {} attestations, updating state.",
            block.body.attestations.len()
        );

        per_block_processing_without_verifying_block_signature(&mut state, &block, &self.spec)?;

        let state_root = state.canonical_root();

        block.state_root = state_root;

        Ok((block, state))
    }

    // TODO: Left this as is, modify later
    pub fn fork_choice(&self) -> Result<(), Error> {
        let present_head = self.finalized_head().beacon_block_root;

        let new_head = self
            .fork_choice
            .write()
            .find_head(&present_head, &self.spec)?;

        if new_head != present_head {
            let block: BeaconBlock = self
                .store
                .get(&new_head)?
                .ok_or_else(|| Error::MissingBeaconBlock(new_head))?;
            let block_root = block.canonical_root();

            let state: BeaconState<E> = self
                .store
                .get(&block.state_root)?
                .ok_or_else(|| Error::MissingBeaconState(block.state_root))?;
            let state_root = state.canonical_root();

            self.update_canonical_head(block, block_root, state.clone(), state_root);

            // Update the canonical `BeaconState`.
            self.update_state(state)?;
        }

        Ok(())
    }

    /// Returns `true` if the given block root has not been processed.
    pub fn is_new_block_root(&self, beacon_block_root: &Hash256) -> Result<bool, Error> {
        Ok(!self.store.exists::<BeaconBlock>(beacon_block_root)?)
    }

    /// Dumps the entire canonical chain, from the head to genesis to a vector for analysis.
    ///
    /// This could be a very expensive operation and should only be done in testing/analysis
    /// activities.
    pub fn chain_dump(&self) -> Result<Vec<CheckPoint<E>>, Error> {
        let mut dump = vec![];

        let mut last_slot = CheckPoint {
            beacon_block: self.head().beacon_block.clone(),
            beacon_block_root: self.head().beacon_block_root,
            beacon_state: self.head().beacon_state.clone(),
            beacon_state_root: self.head().beacon_state_root,
        };

        dump.push(last_slot.clone());

        loop {
            let beacon_block_root = last_slot.beacon_block.previous_block_root;

            if beacon_block_root == self.spec.zero_hash {
                break; // Genesis has been reached.
            }

            let beacon_block: BeaconBlock =
                self.store.get(&beacon_block_root)?.ok_or_else(|| {
                    Error::DBInconsistent(format!("Missing block {}", beacon_block_root))
                })?;
            let beacon_state_root = beacon_block.state_root;
            let beacon_state = self.store.get(&beacon_state_root)?.ok_or_else(|| {
                Error::DBInconsistent(format!("Missing state {}", beacon_state_root))
            })?;

            let slot = CheckPoint {
                beacon_block,
                beacon_block_root,
                beacon_state,
                beacon_state_root,
            };

            dump.push(slot.clone());
            last_slot = slot;
        }

        dump.reverse();

        Ok(dump)
    }
}

impl From<DBError> for Error {
    fn from(e: DBError) -> Error {
        Error::DBError(e)
    }
}

impl From<ForkChoiceError> for Error {
    fn from(e: ForkChoiceError) -> Error {
        Error::ForkChoiceError(e)
    }
}

impl From<BeaconStateError> for Error {
    fn from(e: BeaconStateError) -> Error {
        Error::BeaconStateError(e)
    }
}
