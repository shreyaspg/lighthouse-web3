use super::{success_response, ApiResult};
use crate::{helpers::*, ApiError, UrlQuery};
use beacon_chain::{BeaconChain, BeaconChainTypes};
use hyper::{Body, Request};
use std::sync::Arc;
use store::Store;
use types::{BeaconBlock, BeaconState};

/// HTTP handler to return a `BeaconBlock` at a given `root` or `slot`.
pub fn get_best_slot<T: BeaconChainTypes + 'static>(req: Request<Body>) -> ApiResult {
    let beacon_chain = req
        .extensions()
        .get::<Arc<BeaconChain<T>>>()
        .ok_or_else(|| ApiError::ServerError("Beacon chain extension missing".to_string()))?;

    let slot = beacon_chain.head().beacon_state.slot;

    let json: String = serde_json::to_string(&slot)
        .map_err(|e| ApiError::ServerError(format!("Unable to serialize Slot: {:?}", e)))?;

    Ok(success_response(Body::from(json)))
}

/// HTTP handler to return a `BeaconBlock` at a given `root` or `slot`.
pub fn get_block<T: BeaconChainTypes + 'static>(req: Request<Body>) -> ApiResult {
    let beacon_chain = req
        .extensions()
        .get::<Arc<BeaconChain<T>>>()
        .ok_or_else(|| ApiError::ServerError("Beacon chain extension missing".to_string()))?;

    let query_params = ["root", "slot"];
    let (key, value) = UrlQuery::from_request(&req)?.first_of(&query_params)?;

    let block_root = match (key.as_ref(), value) {
        ("slot", value) => {
            let target = parse_slot(&value)?;

            beacon_chain
                .rev_iter_block_roots()
                .take_while(|(_root, slot)| *slot >= target)
                .find(|(_root, slot)| *slot == target)
                .map(|(root, _slot)| root)
                .ok_or_else(|| {
                    ApiError::NotFound(format!("Unable to find BeaconBlock for slot {}", target))
                })?
        }
        ("root", value) => parse_root(&value)?,
        _ => return Err(ApiError::ServerError("Unexpected query parameter".into())),
    };

    let block = beacon_chain
        .store
        .get::<BeaconBlock<T::EthSpec>>(&block_root)?
        .ok_or_else(|| {
            ApiError::NotFound(format!(
                "Unable to find BeaconBlock for root {}",
                block_root
            ))
        })?;

    let json: String = serde_json::to_string(&block)
        .map_err(|e| ApiError::ServerError(format!("Unable to serialize BeaconBlock: {:?}", e)))?;

    Ok(success_response(Body::from(json)))
}

/// HTTP handler to return a `BeaconBlock` root at a given `slot`.
pub fn get_block_root<T: BeaconChainTypes + 'static>(req: Request<Body>) -> ApiResult {
    let beacon_chain = req
        .extensions()
        .get::<Arc<BeaconChain<T>>>()
        .ok_or_else(|| ApiError::ServerError("Beacon chain extension missing".to_string()))?;

    let slot_string = UrlQuery::from_request(&req)?.only_one("slot")?;
    let target = parse_slot(&slot_string)?;

    let root = beacon_chain
        .rev_iter_block_roots()
        .take_while(|(_root, slot)| *slot >= target)
        .find(|(_root, slot)| *slot == target)
        .map(|(root, _slot)| root)
        .ok_or_else(|| {
            ApiError::NotFound(format!("Unable to find BeaconBlock for slot {}", target))
        })?;

    let json: String = serde_json::to_string(&root)
        .map_err(|e| ApiError::ServerError(format!("Unable to serialize root: {:?}", e)))?;

    Ok(success_response(Body::from(json)))
}

/// HTTP handler to return a `BeaconState` at a given `root` or `slot`.
///
/// Will not return a state if the request slot is in the future. Will return states higher than
/// the current head by skipping slots.
pub fn get_state<T: BeaconChainTypes + 'static>(req: Request<Body>) -> ApiResult {
    let beacon_chain = req
        .extensions()
        .get::<Arc<BeaconChain<T>>>()
        .ok_or_else(|| ApiError::ServerError("Beacon chain extension missing".to_string()))?;

    let query_params = ["root", "slot"];
    let (key, value) = UrlQuery::from_request(&req)?.first_of(&query_params)?;

    let state: BeaconState<T::EthSpec> = match (key.as_ref(), value) {
        ("slot", value) => state_at_slot(&beacon_chain, parse_slot(&value)?)?,
        ("root", value) => {
            let root = &parse_root(&value)?;

            beacon_chain
                .store
                .get(root)?
                .ok_or_else(|| ApiError::NotFound(format!("No state for root: {}", root)))?
        }
        _ => return Err(ApiError::ServerError("Unexpected query parameter".into())),
    };

    let json: String = serde_json::to_string(&state)
        .map_err(|e| ApiError::ServerError(format!("Unable to serialize BeaconState: {:?}", e)))?;

    Ok(success_response(Body::from(json)))
}

/// HTTP handler to return a `BeaconState` root at a given `slot`.
///
/// Will not return a state if the request slot is in the future. Will return states higher than
/// the current head by skipping slots.
pub fn get_state_root<T: BeaconChainTypes + 'static>(req: Request<Body>) -> ApiResult {
    let beacon_chain = req
        .extensions()
        .get::<Arc<BeaconChain<T>>>()
        .ok_or_else(|| ApiError::ServerError("Beacon chain extension missing".to_string()))?;

    let slot_string = UrlQuery::from_request(&req)?.only_one("slot")?;
    let slot = parse_slot(&slot_string)?;

    let root = state_root_at_slot(&beacon_chain, slot)?;

    let json: String = serde_json::to_string(&root)
        .map_err(|e| ApiError::ServerError(format!("Unable to serialize root: {:?}", e)))?;

    Ok(success_response(Body::from(json)))
}

/// HTTP handler to return the highest finalized slot.
pub fn get_latest_finalized_checkpoint<T: BeaconChainTypes + 'static>(
    req: Request<Body>,
) -> ApiResult {
    let beacon_chain = req
        .extensions()
        .get::<Arc<BeaconChain<T>>>()
        .ok_or_else(|| ApiError::ServerError("Beacon chain extension missing".to_string()))?;

    let checkpoint = beacon_chain
        .head()
        .beacon_state
        .finalized_checkpoint
        .clone();

    let json: String = serde_json::to_string(&checkpoint)
        .map_err(|e| ApiError::ServerError(format!("Unable to serialize checkpoint: {:?}", e)))?;

    Ok(success_response(Body::from(json)))
}
