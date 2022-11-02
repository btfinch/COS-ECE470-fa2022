use crate::miner::new;
use crate::types::block::{Block, generate_random_block_1};
use crate::types::hash::{H256, Hashable};
use std::collections::HashMap;
use hex_literal::hex;

use crate::types::{merkle::MerkleTree, transaction::SignedTransaction,block::{Header,Content}};




pub struct Blockchain {
    pub map: HashMap<H256, Block>,
    // Additional hashmap to store the level number of each block with their hash
    pub level_map: HashMap<H256, u64>,
    pub tip_hash: H256,
    pub tip_level: u64, //genesis, level 0
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        let zeros: [u8; 32] = [0;32];
        let parent: H256 = H256::from(zeros);
        
        let mut new_map = HashMap::new();
        let noncy: u32 = 1;
        let dify: H256 = hex!("00004fffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").into();
        let timy: u128 = 0;
        let empty: Vec<H256> = Vec::new();
        let merkly = MerkleTree::new(&empty).root();
        let heady= Header{parent: parent, nonce: noncy, difficulty: dify, timestamp: timy, merkle_root: merkly};
        let vec:Vec<SignedTransaction> = Vec::new();
        let no_content = Content(vec);
        let genesis = Block{header: heady, content: no_content};
        let genesis_hash = genesis.hash();
        let genesis_hash_copy = genesis_hash.clone();
        new_map.insert(genesis_hash, genesis);

        let level: u64 = 0;

        let mut new_level_map: HashMap<H256,u64> = HashMap::new();
        new_level_map.insert(genesis_hash_copy,0);
        Self {map: new_map, level_map: new_level_map, tip_hash: genesis_hash, tip_level: level}
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        let block_hash = block.hash();
        let block_hash_copy = block_hash.clone();
        let block_hash_copy2 = block_hash.clone();
        let genesis_level: u64 = 0;
        let mut parent_level = genesis_level;
        let block_copy_1 = block.clone();
        let parent_hash = block_copy_1.header.parent.clone();
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
}

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