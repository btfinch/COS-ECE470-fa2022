use serde::{Serialize, Deserialize};
use rand;
use crate::types::hash::{H256, Hashable};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{merkle::MerkleTree, transaction::SignedTransaction};

#[derive(Serialize, Deserialize, Debug, Clone)]



pub struct Block {
    pub header: Header,
    pub content: Content,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header{

    pub parent: H256,
    pub nonce: u32,
    pub difficulty: H256,
    pub timestamp: u128,
    pub merkle_root: H256,
} 


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content(pub Vec<SignedTransaction>);

impl Hashable for Header{
    fn hash(&self) -> H256 {
        let serial_signed = serde_json::to_string(self); 
        ring::digest::digest(&ring::digest::SHA256, serial_signed.unwrap().as_bytes()).into()
    }
}

impl Hashable for Block {
    fn hash(&self) -> H256 {
        self.header.hash()
    }
}

impl Block {
    pub fn get_parent(&self) -> H256 {
        self.header.parent
    }

    pub fn get_difficulty(&self) -> H256 {
        self.header.difficulty
    }
}

pub fn generate_random_block_1(parent: &H256) -> Block {
    let noncy: u32 = rand::random();
    let dify: H256 = [255u8; 32].into();
    let timy = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let empty: Vec<H256> = Vec::new();
    let merkly = MerkleTree::new(&empty).root();
    let heady= Header{parent: *parent, nonce: noncy, difficulty: dify, timestamp: timy, merkle_root: merkly};
    let vec:Vec<SignedTransaction> = Vec::new();
    let no_content = Content(vec);
    let lev: u64 = 0;
    let block = Block{header: heady, content: no_content};
    block


}

pub fn generate_random_block_2(parent: &H256, difficulty: &H256) -> Block {
    let noncy: u32 = rand::random();
    let timy = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let empty: Vec<H256> = Vec::new();
    let merkly = MerkleTree::new(&empty).root();
    let heady= Header{parent: *parent, nonce: noncy, difficulty: *difficulty, timestamp: timy, merkle_root: merkly};
    let vec:Vec<SignedTransaction> = Vec::new();
    let no_content = Content(vec);
    let block = Block{header: heady, content: no_content};
    block


}

#[cfg(any(test, test_utilities))]
pub fn generate_random_block(parent: &H256) -> Block {
    let noncy: u32 = rand::random();
    let dify: H256 = [255u8; 32].into();
    let timy = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    let empty: Vec<H256> = Vec::new();
    let merkly = MerkleTree::new(&empty).root();
    let heady= Header{parent: *parent, nonce: noncy, difficulty: dify, timestamp: timy, merkle_root: merkly};
    let vec:Vec<SignedTransaction> = Vec::new();
    let no_content = Content(vec);
    let lev: u64 = 0;
    let block = Block{header: heady, content: no_content};
    block


}