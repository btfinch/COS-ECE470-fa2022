use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use log::{debug, info};
use crate::types::block::Block;
use crate::network::server::Handle as ServerHandle;
use crate::types::hash::{H256, Hashable};
use std::thread;
use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use crate::network::worker;
use crate::network::message::Message;


#[derive(Clone)]
pub struct Worker {
    server: ServerHandle,
    finished_block_chan: Receiver<Block>,
    blockchain: Arc<Mutex<Blockchain>>,
}

impl Worker {
    pub fn new(
        server: &ServerHandle,
        finished_block_chan: Receiver<Block>,
        blockchain: &Arc<Mutex<Blockchain>>,
    ) -> Self {
        Self {
            server: server.clone(),
            finished_block_chan,
            blockchain: Arc::clone(blockchain),
        }
    }

    pub fn start(self) {
        thread::Builder::new()
            .name("miner-worker".to_string())
            .spawn(move || {
                self.worker_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn worker_loop(&self) {
        loop {
            let _block = self.finished_block_chan.recv().expect("Receive finished block error");
            // print!("worker recieved block");
            let new_block = _block.clone();
            // TODO for student: insert this finished block to blockchain, and broadcast this block hash
            {
                self.blockchain.lock().unwrap().insert(&_block);
            }
            
            let mut block_vec: Vec<H256> = Vec::new();
            block_vec.push(new_block.hash());
            //print hash of tip
            {
                println!("{:?}",self.blockchain.lock().unwrap().tip());
            }
            self.server.broadcast(Message::NewBlockHashes(block_vec));
            
        }
    }
}
