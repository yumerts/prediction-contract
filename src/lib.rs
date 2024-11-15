//!
//! Stylus Prediction Contract
//! 
//! This contract allows users to place predictions on running matches

// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use alloy_primitives::Address;
use alloy_sol_types::sol;
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{alloy_primitives::U256, msg, prelude::*, storage::{StorageAddress, StorageBool, StorageMap, StorageU256, StorageVec}};

const USDC_TOKEN_ADDRESS : Address = address!("75faf114eafb1BDbe2F0316DF893fd58CE46AA4d"); //USDC Testnet Token Address

sol!{
    event prediction_results_available(uint256 indexed match_id, bool result);
}

sol_interface! {
    interface IERC20 {
        function totalSupply() external view returns (uint256);
        function balanceOf(address account) external view returns (uint256);
        function transfer(address recipient, uint256 amount)
            external
            returns (bool);
        function allowance(address owner, address spender)
            external
            view
            returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address sender, address recipient, uint256 amount)
            external
            returns (bool);
    }
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
    player_info_smart_contract_address: StorageAddress,
    match_info_smart_contract_address: StorageAddress,
    prediction_pools: StorageMap<U256, PredictionPool> //Match ID -> Prediction Pool
}

#[public]
impl PredictionContract{
    fn init(&mut self) -> Result<(), Vec<u8>>{
        let initialized = self.initialized.get();
        if initialized{
            return Err("Already initialized".into());
        }

        self.initialized.set(true);
        self.owner.set(msg::sender());

        Ok(())
    }

    fn get_player_info_smart_contract_address(&self) -> Address{
        self.player_info_smart_contract_address.get()
    }

    fn set_player_info_smart_contract_address(&mut self, player_info_smart_contract_address: Address) -> Result<(), Vec<u8>>{
        self.player_info_smart_contract_address.set(player_info_smart_contract_address);
        Ok(())
    }

    fn get_match_info_smart_contract_address(&self) -> Address{
        self.match_info_smart_contract_address.get()
    }

    fn set_match_info_smart_contract_address(&mut self, match_info_smart_contract_address: Address) -> Result<(), Vec<u8>>{
        self.match_info_smart_contract_address.set(match_info_smart_contract_address);
        Ok(())
    }

    // Match Info Smart Contract Triggers this function so other people can view this
    fn create_prediction_pool(&mut self, match_id: U256) -> Result<(), Vec<u8>>{
        let initialized = self.initialized.get();
        if !initialized{
            return Err("The contract has not been initialized just yet".into());
        }

        let match_info_smart_contract_address = self.match_info_smart_contract_address.get();
        if msg::sender() != match_info_smart_contract_address{
            return Err("Only the match info smart contract can create a prediction pool".into());
        }

        let mut prediction_pool_setter = self.prediction_pools.setter(match_id);
        prediction_pool_setter.exists.set(true);
        prediction_pool_setter.player1_pool_stake.set(U256::from(0));
        prediction_pool_setter.player2_pool_stake.set(U256::from(0));
        prediction_pool_setter.total_staked.set(U256::from(0));

        Ok(())
    }
}
