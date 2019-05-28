use super::errors::{ExitInvalid as Invalid, ExitValidationError as Error};
use tree_hash::SignedRoot;
use types::*;

/// Indicates if an `Exit` is valid to be included in a block in the current epoch of the given
/// state.
///
/// Returns `Ok(())` if the `Exit` is valid, otherwise indicates the reason for invalidity.
///
/// Spec v0.5.1
pub fn verify_exit<T: EthSpec>(
    state: &BeaconState<T>,
    exit: &VoluntaryExit,
    spec: &ChainSpec,
) -> Result<(), Error> {
    verify_exit_parametric(state, exit, spec, false)
}

/// Like `verify_exit` but doesn't run checks which may become true in future states.
pub fn verify_exit_time_independent_only<T: EthSpec>(
    state: &BeaconState<T>,
    exit: &VoluntaryExit,
    spec: &ChainSpec,
) -> Result<(), Error> {
    verify_exit_parametric(state, exit, spec, true)
}

/// Parametric version of `verify_exit` that skips some checks if `time_independent_only` is true.
fn verify_exit_parametric<T: EthSpec>(
    state: &BeaconState<T>,
    exit: &VoluntaryExit,
    spec: &ChainSpec,
    time_independent_only: bool,
) -> Result<(), Error> {
    let validator = state
        .validator_registry
        .get(exit.validator_index as usize)
        .ok_or_else(|| Error::Invalid(Invalid::ValidatorUnknown(exit.validator_index)))?;

    // Verify that the validator has not yet exited.
    verify!(
        validator.exit_epoch == spec.far_future_epoch,
        Invalid::AlreadyExited(exit.validator_index)
    );

    // Verify that the validator has not yet initiated.
    verify!(
        !validator.initiated_exit,
        Invalid::AlreadyInitiatedExited(exit.validator_index)
    );

    // Exits must specify an epoch when they become valid; they are not valid before then.
    verify!(
        time_independent_only || state.current_epoch(spec) >= exit.epoch,
        Invalid::FutureEpoch {
            state: state.current_epoch(spec),
            exit: exit.epoch
        }
    );

    // Must have been in the validator set long enough.
    let lifespan = state.slot.epoch(spec.slots_per_epoch) - validator.activation_epoch;
    verify!(
        lifespan >= spec.persistent_committee_period,
        Invalid::TooYoungToLeave {
            lifespan,
            expected: spec.persistent_committee_period,
        }
    );

    let message = exit.signed_root();
    let domain = spec.get_domain(exit.epoch, Domain::Exit, &state.fork);

    verify!(
        exit.signature
            .verify(&message[..], domain, &validator.pubkey),
        Invalid::BadSignature
    );

    Ok(())
}
