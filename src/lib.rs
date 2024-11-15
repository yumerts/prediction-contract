//!
//! Stylus Prediction Contract
//! 
//! This contract allows users to place predictions on running matches

// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use alloy_sol_types::sol;
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{alloy_primitives::U256, prelude::*, storage::{StorageAddress, StorageBool, StorageU256, StorageVec}};

sol!{
    event prediction_results_available(uint256 indexed match_id, bool result);
}

#[storage]
struct PredictionPool{
    exists: StorageBool,

    player1_predictor_count: StorageU256,
    player1_predictor : StorageVec<StorageAddress>,
    player1_predictor_stake : StorageVec<StorageU256>,
    player1_pool_stake: StorageU256,

    player2_predictor_count: StorageU256,
    player2_predictor : StorageVec<StorageAddress>,
    player2_predictor_stake : StorageVec<StorageU256>,
    player2_pool_stake: StorageU256,

    total_staked: StorageU256
}

#[storage]
#[entrypoint]
pub struct PredictionContract{
    initialized: StorageBool,
    owner: StorageAddress,
}

#[public]
impl PredictionContract{
    
}
