use beacon_chain::{BeaconChain, BeaconChainError, BeaconChainTypes};
use eth2::lighthouse::{ValidatorPerformance, ValidatorPerformanceQuery};
use state_processing::{
    per_epoch_processing::{
        altair::participation_cache::Error as ParticipationCacheError, EpochProcessingSummary,
    },
    BlockReplayError, BlockReplayer,
};
use std::sync::Arc;
use types::{BeaconState, BeaconStateError, EthSpec, Hash256, SignedBeaconBlock, Slot};
use warp_utils::reject::{beacon_chain_error, custom_bad_request, custom_server_error};

const BLOCK_ROOT_CHUNK_SIZE: usize = 100;

#[derive(Debug)]
enum ValidatorReportError {
    BlockReplay(BlockReplayError),
    BeaconState(BeaconStateError),
    ParticipationCache(ParticipationCacheError),
    MissingValidator(usize),
}

impl From<BlockReplayError> for ValidatorReportError {
    fn from(e: BlockReplayError) -> Self {
        Self::BlockReplay(e)
    }
}

impl From<BeaconStateError> for ValidatorReportError {
    fn from(e: BeaconStateError) -> Self {
        Self::BeaconState(e)
    }
}

impl From<ParticipationCacheError> for ValidatorReportError {
    fn from(e: ParticipationCacheError) -> Self {
        Self::ParticipationCache(e)
    }
}

pub fn get_validator_performance<T: BeaconChainTypes>(
    query: ValidatorPerformanceQuery,
    chain: Arc<BeaconChain<T>>,
) -> Result<Vec<ValidatorPerformance>, warp::Rejection> {
    let spec = &chain.spec;

    let start_epoch = query.start_epoch;
    let start_slot = start_epoch.start_slot(T::EthSpec::slots_per_epoch());
    let end_epoch = query.end_epoch;
    // Add 1 to `end_slot` so a summary is generated for `end_epoch`.
    let end_slot = end_epoch.end_slot(T::EthSpec::slots_per_epoch()) + 1;

    // Check query is valid
    if start_epoch > end_epoch || start_epoch == 0 {
        return Err(custom_bad_request(format!(
            "invalid start and end: {}, {}",
            start_epoch, end_epoch
        )));
    }

    let head_state = chain.head_beacon_state().map_err(beacon_chain_error)?;

    // Load block roots.
    let mut block_roots: Vec<Hash256> = chain
        .forwards_iter_block_roots_until(start_slot, end_slot)
        .map_err(beacon_chain_error)?
        .collect::<Result<Vec<(Hash256, Slot)>, _>>()
        .map_err(beacon_chain_error)?
        .iter()
        .map(|(root, _)| *root)
        .collect();
    block_roots.dedup();

    // Load state for block replay
    let starting_state_root = chain
        .state_root_at_slot(start_slot)
        .and_then(|maybe_root| {
            maybe_root.ok_or(BeaconChainError::UnableToFindTargetRoot(start_slot))
        })
        .map_err(beacon_chain_error)?;

    let starting_state = chain
        .get_state(&starting_state_root, Some(start_slot))
        .and_then(|maybe_state| {
            maybe_state.ok_or(BeaconChainError::MissingBeaconState(starting_state_root))
        })
        .map_err(beacon_chain_error)?;

    // Allocate ValidatorPerformance vector for each validator.
    let mut perfs = ValidatorPerformance::initialize(head_state.validators().len());

    let post_slot_hook = |_state: &mut BeaconState<T::EthSpec>,
                          summary: Option<EpochProcessingSummary<T::EthSpec>>,
                          _is_skip_slot: bool|
     -> Result<(), ValidatorReportError> {
        if let Some(summary) = summary {
            // Iterate through the performances of each validator.
            for index in 0..perfs.len() {
                let perf = perfs
                    .get_mut(index)
                    .ok_or(ValidatorReportError::MissingValidator(index))?;

                if summary.is_active_unslashed_in_previous_epoch(index) {
                    if perf.always_active.is_none() {
                        perf.always_active = Some(true);
                    }

                    if summary.is_previous_epoch_source_attester(index)? {
                        perf.source_attestation_hits += 1;
                    } else {
                        perf.source_attestation_misses += 1;
                    }

                    if summary.is_previous_epoch_head_attester(index)? {
                        perf.head_attestation_hits += 1;
                    } else {
                        perf.head_attestation_misses += 1;
                    }

                    if summary.is_previous_epoch_target_attester(index)? {
                        perf.target_attestation_hits += 1;
                    } else {
                        perf.target_attestation_misses += 1;
                    }

                    if let Some(inclusion_info) = summary.previous_epoch_inclusion_info(index) {
                        *perf.delays.entry(inclusion_info.delay).or_default() += 1
                    }
                } else {
                    perf.always_active = Some(false);
                }
            }
        }

        Ok(())
    };

    // Build BlockReplayer.
    let mut replayer = BlockReplayer::new(starting_state, spec)
        .no_state_root_iter()
        .no_signature_verification()
        .minimal_block_root_verification()
        .post_slot_hook(Box::new(post_slot_hook));

    // Iterate through the block roots, loading blocks in chunks to reduce load on memory.
    for block_root_chunks in block_roots
        .chunks(BLOCK_ROOT_CHUNK_SIZE)
        .take((end_slot - start_slot).as_usize() + 1)
    {
        // Load blocks from the block root chunks.
        let blocks = block_root_chunks
            .iter()
            .map(|root| {
                chain
                    .get_block(root)
                    .and_then(|maybe_block| {
                        maybe_block.ok_or(BeaconChainError::MissingBeaconBlock(*root))
                    })
                    .map_err(beacon_chain_error)
            })
            .collect::<Result<Vec<SignedBeaconBlock<T::EthSpec>>, _>>()?;

        replayer = replayer
            .apply_blocks(blocks, None)
            .map_err(|e: ValidatorReportError| custom_server_error(format!("{:?}", e)))?;
    }

    drop(replayer);

    Ok(perfs)
}
