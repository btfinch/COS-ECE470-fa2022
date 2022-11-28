use crate::miner::new;
use crate::types::address::Address;
use crate::types::block::{Block, generate_random_block_1, self};
use crate::types::hash::{H256, Hashable};
use std::collections::HashMap;
use hex_literal::hex;
use ring::rand::generate;
use ring::signature::{Ed25519KeyPair, KeyPair};

use crate::types::{merkle::MerkleTree, transaction::SignedTransaction,block::{Header,Content}};




pub struct Blockchain {
    pub map: HashMap<H256, Block>,
    // Additional hashmap to store the level number of each block with their hash
    pub level_map: HashMap<H256, u64>,
    pub tip_hash: H256,
    pub tip_level: u64, //genesis, level 0

    pub state_map: HashMap<H256, HashMap<Address, (u32, u32)>> // format: (account_nonce, balance)
}

pub struct Mempool{
    pub map: HashMap<H256, SignedTransaction>,
}


impl Mempool{
    pub fn new() -> Self{
        let mut new_map = HashMap::new();
        Self {map: new_map}
    }
    pub fn insert(&mut self, st: &SignedTransaction) {
        let st_hash = st.clone().hash();
        self.map.insert(st_hash,st.clone());
    }
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        let zeros: [u8; 32] = [0;32];
        let parent: H256 = H256::from(zeros);
        
        let mut new_map = HashMap::new();
        let noncy: u32 = 1;
        let dify: H256 = hex!("000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").into();
        let timy: u128 = 0;
        let empty: Vec<H256> = Vec::new();
        let merkly = MerkleTree::new(&empty).root();
        let heady= Header{parent: parent, nonce: noncy, difficulty: dify, timestamp: timy, merkle_root: merkly};
        let vec:Vec<SignedTransaction> = Vec::new();
        let no_content = Content(vec);
        let genesis = Block{header: heady, content: no_content};
        let genesis_hash = genesis.hash();
        let genesis_hash_copy = genesis_hash.clone();
        let genesis_hash_copy_2 = genesis_hash.clone();
        new_map.insert(genesis_hash, genesis);

        let level: u64 = 0;

        let mut new_level_map: HashMap<H256,u64> = HashMap::new();
        new_level_map.insert(genesis_hash_copy,0);


        //creating state map:
        let mut new_state_map: HashMap<H256, HashMap<Address, (u32, u32)>> =HashMap::new();
        
        // initial coin offering: assigning initial coins: 9 accounts total, 3 for each node, only 1 has 1073741824 coins
        let mut genesis_state: HashMap<Address, (u32, u32)> = HashMap::new();

        let mut address_vec: Vec<Address> = Vec::new();

        // (one account with 1073741824 coins)

        let kp_seed = &[0;32];
        let key_pair = Ed25519KeyPair::from_seed_unchecked(kp_seed).unwrap();
        let public_key_bytes = key_pair.public_key();
        let produced_public_key = public_key_bytes.clone().as_ref().to_vec();
        let generated = Address::from_public_key_bytes(public_key_bytes.as_ref());
        // order: (account_nonce, balance) (alphabetical)
        genesis_state.insert(generated,(0,1073741824));
        /* 
        for i in 1..9{ // 8 accounts with 0 coins
            let kp_seed = &[i;32];
            let key_pair = Ed25519KeyPair::from_seed_unchecked(kp_seed).unwrap();
            let public_key_bytes = key_pair.public_key();
            let produced_public_key = public_key_bytes.clone().as_ref().to_vec();
            let generated = Address::from_public_key_bytes(public_key_bytes.as_ref());
            genesis_state.insert(generated,(0,0));

        } */
        
        new_state_map.insert(genesis_hash_copy_2,genesis_state);


        Self {map: new_map, level_map: new_level_map, tip_hash: genesis_hash, tip_level: level, state_map: new_state_map}
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        let block_hash = block.hash();
        let block_hash_copy = block_hash.clone();
        let block_hash_copy2 = block_hash.clone();
        let block_hash_copy3 = block_hash.clone();
        let genesis_level: u64 = 0;
        let mut parent_level = genesis_level;
        let block_copy_1 = block.clone();
        let parent_hash = block_copy_1.header.parent.clone();
        let parent_hash2 = parent_hash.clone();
        match self.level_map.get(&parent_hash){
            Some(parent_lev) => parent_level = parent_lev.clone(), // If code is not working check this line
            _=> println!("Invalid Parent Hash"),
        }
        let block_level = parent_level+1;
        if block_level > self.tip_level +1{
            println!("Block Level is too big! (>1 + tip_level)")
        }
        if block_level > self.tip_level{
            self.tip_level = self.tip_level+1;
            self.tip_hash = block_hash_copy2;
        }
        

        self.level_map.insert(block_hash_copy,block_level);


        let mut block_state: HashMap<Address, (u32, u32)> = self.state_map.get(&parent_hash2).unwrap().clone();
        // go through previous state and change the values in any account in which there was a transaction:
        let block_details = block.get_transaction_details();

        for i in block_details.into_iter(){
            let transaction = i.get_transaction();
            let sendery = transaction.get_sender();
            let recievery = transaction.get_reciever();
            let valuey = transaction.get_value();

            if block_state.contains_key(&recievery){
                let (send_an, send_bal) = *block_state.get(&sendery).unwrap();
                let (recieve_an, recieve_bal) = *block_state.get(&recievery).unwrap();
                let new_san = send_an +1;
                if send_bal >= valuey{
                    let new_send_bal = send_bal - valuey.clone();
                    let new_recieve_bal = recieve_bal+valuey.clone();

                    *block_state.get_mut(&sendery).unwrap() = (new_san,new_send_bal); 
                    *block_state.get_mut(&recievery).unwrap() = (recieve_an,new_recieve_bal);
                }
                

            }
            else{
                let (send_an, send_bal) = *block_state.get(&sendery).unwrap();
                let new_san = send_an +1;

                if send_bal >= valuey {
                    let new_send_bal = send_bal - valuey.clone();

                    *block_state.get_mut(&sendery).unwrap() = (new_san,new_send_bal); 
                    block_state.insert(recievery.clone(), (0,valuey.clone()));
                }
                
            }
        }
        
        self.state_map.insert(block_hash_copy3,block_state);
        
        let block_copy = block.clone();

        self.map.insert(block_hash, block_copy);
        // println!("{:?}     ", block_level)


    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        self.tip_hash.clone()
    }

    /// Get all blocks' hashes of the longest chain, ordered from genesis to the tip
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        let mut new_vec: Vec<H256> = vec!();
        let mut temp_hash = self.tip();
        for i in 0..(self.tip_level+1){
            let temp_block = self.map.get(&temp_hash).unwrap();
            let temp_block_copy = temp_block.clone();
            let block_parent_hash = temp_block_copy.header.parent.clone();

            new_vec.insert(0,temp_hash);
            temp_hash = block_parent_hash;
        }
        new_vec
    }

    // Get alll tx hashes in longest chain
    pub fn all_tx_in_longest_chain(&self) -> Vec<Vec<H256>> {
        let block_hashes = self.all_blocks_in_longest_chain();
        let mut outer_vec: Vec<Vec<H256>> = vec!();
        for block in block_hashes.into_iter(){
            let temp_block = self.map.get(&block).unwrap();
            outer_vec.push(temp_block.get_transactions());
        }
        outer_vec
    }

}

/* 
// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;

    #[test]
    fn insert_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        blockchain.insert(&block);
        assert_eq!(blockchain.tip(), block.hash());

    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
*/