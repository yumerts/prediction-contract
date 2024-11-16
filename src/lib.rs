//!
//! Stylus Prediction Contract
//! 
//! This contract allows users to place predictions on running matches

// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use alloy_primitives::{address, Address};
use alloy_sol_types::sol;
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{alloy_primitives::U256, evm, msg, prelude::*, storage::{StorageAddress, StorageBool, StorageMap, StorageU256, StorageVec}};

const USDC_TOKEN_ADDRESS : Address = address!("75faf114eafb1BDbe2F0316DF893fd58CE46AA4d"); //USDC Testnet Token Address

sol!{
    event placed_prediction(uint256 indexed match_id, address indexed predictor, uint256 indexed party, uint256 stake);
    event prediction_results_available(uint256 indexed match_id, uint256 indexed winner);
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
    this_address: StorageAddress,
    player_info_smart_contract_address: StorageAddress,
    match_info_smart_contract_address: StorageAddress,
    prediction_pools: StorageMap<U256, PredictionPool>,  //Match ID -> Prediction Pool
    withdrawable_pool: StorageMap<Address, StorageU256> //User Address -> Withdrawable Amount of USDC after winning predictions
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

    fn get_this_address(&self) -> Address{
        self.this_address.get()
    }

    fn set_this_address(&mut self, this_address: Address) -> Result<(), Vec<u8>>{
        self.this_address.set(this_address);
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

    // predict match function
    // allows the user to stake USDC token on a match
    #[payable]
    fn predict_match(&mut self, match_id: U256, party: U256, usdc_amount: U256) -> Result<(), Vec<u8>>{
        let initialized  = self.initialized.get();
        if !initialized {
            return Err("The contract has not been initialized".into());
        }

        let match_info_smart_contract_address = self.match_info_smart_contract_address.get();
        if msg::sender() != match_info_smart_contract_address{
            return Err("Only the match info smart contract can create a prediction pool".into());
        }

        if party != U256::from(1) && party != U256::from(2) {
            return Err("Invalid party".into());
        }

        let match_pool = self.prediction_pools.get(match_id);
        if !match_pool.exists.get() {
            return Err("The match does not exist".into());
        }

        for i in 0..match_pool.player1_predictor.len() {
            if match_pool.player1_predictor.get(i).unwrap() == msg::sender() {
                return Err("You have already predicted in this match".into());
            }
        }

        for i in 0..match_pool.player2_predictor.len() {
            if match_pool.player2_predictor.get(i).unwrap() == msg::sender() {
                return Err("You have already predicted in this match".into());
            }
        }

        let this_address = self.this_address.get();
        let player1_pool_stake = match_pool.player1_pool_stake.get();
        let player2_pool_stake = match_pool.player2_pool_stake.get();
        let old_staked_amount = match_pool.total_staked.get();
        let player1_predictor_count = match_pool.player1_predictor_count.get();
        let player2_predictor_count = match_pool.player2_predictor_count.get();
        drop(match_pool);

        let mut match_pool_setter = self.prediction_pools.setter(match_id);
        
        match_pool_setter.total_staked.set(old_staked_amount + usdc_amount);
        if party == U256::from(1){
            match_pool_setter.player1_predictor.push(msg::sender());
            match_pool_setter.player1_predictor_stake.push(usdc_amount);
            match_pool_setter.player1_pool_stake.set(player1_pool_stake + usdc_amount);
            match_pool_setter.player1_predictor_count.set(player1_predictor_count + U256::from(1));
        }else{
            match_pool_setter.player2_predictor.push(msg::sender());
            match_pool_setter.player2_predictor_stake.push(usdc_amount);
            match_pool_setter.player2_pool_stake.set(player2_pool_stake + usdc_amount);
            match_pool_setter.player2_predictor_count.set(player2_predictor_count + U256::from(1));
        }

        
        // Transfer USDC token from the user to the contract
        //let mut self_reference = &*self;
        //put at the last because transfer_from consume everything
        let transfer_result = IERC20::new(USDC_TOKEN_ADDRESS).transfer_from(self, msg::sender(), this_address, usdc_amount);
        if transfer_result.is_err(){
           return Err("USDC Staking has failed. Prediction Placement has been reverted".into());
           //if payment fails, the prediction will be reverted
        }

        evm::log(
            placed_prediction{
                match_id,
                predictor: msg::sender(),
                party,
                stake: usdc_amount
            }
        );

        Ok(())
    }

    // submit match result function
    // allows the match info smart contract to submit the match result
    // and distribute the rewards to the winners
    // the rewards are distributed based on the proportion of the total stake
    // it will be put into a withdrawal pool where the user can request for withdrawal as calling transfer_usdc is somehow not working for continuously loop
    fn submit_match_result(&mut self, match_id: U256, winner: U256) -> Result<(), Vec<u8>>{
        let initialized = self.initialized.get();
        if !initialized {
            return Err("The contract has not been initialized just yet".into());
        }

        let match_info_smart_contract_address = self.match_info_smart_contract_address.get();
        if msg::sender() != match_info_smart_contract_address{
            return Err("Only the match info smart contract can submit the match result".into());
        }

        let match_pool = self.prediction_pools.get(match_id);
        if !match_pool.exists.get() {
            return Err("The match does not exist".into());
        }

        if winner != U256::from(1) && winner != U256::from(2) {
            return Err("Invalid winner".into());
        }

        let total_staked = match_pool.total_staked.get();
        drop(match_pool);
        
        if winner == U256::from(1) {     
            let winner_total_staked = self.prediction_pools.get(match_id).player1_pool_stake.get();
            for i in 0..self.prediction_pools.get(match_id).player1_predictor.len() {
                let predictor = self.prediction_pools.get(match_id).player1_predictor.get(i).unwrap();
                let predictor_stake = self.prediction_pools.get(match_id).player1_predictor_stake.get(i).unwrap();
                let reward = (predictor_stake * total_staked) / winner_total_staked;

                let old_withdrawable_amount = self.withdrawable_pool.get(predictor);
                let mut withdrawable_pool_setter = self.withdrawable_pool.setter(predictor);
                withdrawable_pool_setter.set(old_withdrawable_amount + reward);
            }
        }else{
            let winner_total_staked = self.prediction_pools.get(match_id).player2_pool_stake.get();
            for i in 0..self.prediction_pools.get(match_id).player2_predictor.len() {
                let predictor = self.prediction_pools.get(match_id).player2_predictor.get(i).unwrap();
                let predictor_stake = self.prediction_pools.get(match_id).player2_predictor_stake.get(i).unwrap();
                let reward = (predictor_stake * total_staked) / winner_total_staked;
                
                let old_withdrawable_amount = self.withdrawable_pool.get(predictor);
                let mut withdrawable_pool_setter = self.withdrawable_pool.setter(predictor);
                withdrawable_pool_setter.set(old_withdrawable_amount + reward);
            }
        }

        evm::log(
            prediction_results_available{
                match_id,
                winner
            }
        );
        Ok(())

    }

    fn withdraw_rewards(&mut self) -> Result<(), Vec<u8>>{
        let initialized = self.initialized.get();
        if !initialized {
            return Err("The contract has not been initialized just yet".into());
        }

        let withdrawable_amount = self.withdrawable_pool.get(msg::sender());
        if withdrawable_amount == U256::from(0){
            return Err("No rewards to withdraw".into());
        }

        let mut withdrawable_pool_setter = self.withdrawable_pool.setter(msg::sender());
        withdrawable_pool_setter.set(U256::from(0));

        let transfer_result = IERC20::new(USDC_TOKEN_ADDRESS).transfer(self, msg::sender(), withdrawable_amount);
        if transfer_result.is_err(){
            return Err("Transfer failed".into());
        }

        Ok(())
    }
}
