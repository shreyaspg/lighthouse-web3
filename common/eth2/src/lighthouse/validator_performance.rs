use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use types::Epoch;

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct ValidatorPerformance {
    /// The index of the queried validator.
    pub validator_index: u64,
    /// The validator had an attestation included on-chain.
    pub source_attestation_hits: usize,
    /// Inverse of `attestation_hits`.
    pub source_attestation_misses: usize,
    /// The validator had an attestation included on-chain which matched the "head" vote.
    pub head_attestation_hits: usize,
    /// Inverse of `head_attestation_hits`.
    pub head_attestation_misses: usize,
    /// The validator had an attestation included on-chain which matched the "target" vote.
    pub target_attestation_hits: usize,
    /// Inverse of `target_attestation_hits`.
    pub target_attestation_misses: usize,
    /// Set to `Some(true)` if the validator was active (i.e., eligible to attest) in all observed
    /// states.
    pub always_active: Option<bool>,
    /// A map of `inclusion_distance -> count`, indicating how many times the validator achieved
    /// each inclusion distance.
    pub delays: HashMap<u64, u64>,
}

impl ValidatorPerformance {
    pub fn initialize(len: usize) -> Vec<Self> {
        let mut vec = vec![];
        for index in 0..len {
            vec.push(Self {
                validator_index: index as u64,
                ..Default::default()
            })
        }
        vec
    }
}

/// Query parameters for the `/lighthouse/validator_performance` endpoint.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ValidatorPerformanceQuery {
    /// Lower slot limit for block rewards returned (inclusive).
    pub start_epoch: Epoch,
    /// Upper slot limit for block rewards returned (inclusive).
    pub end_epoch: Epoch,
}
