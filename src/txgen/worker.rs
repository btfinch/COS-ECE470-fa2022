use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use log::{debug, info};
use crate::types::block::Block;
use crate::network::server::Handle as ServerHandle;
use crate::types::hash::{H256, Hashable};
use std::thread;
use std::sync::{Arc, Mutex};
use crate::blockchain::{Mempool, Blockchain};
use crate::network::worker;
use crate::network::message::Message;
use crate::types::transaction::{SignedTransaction};


#[derive(Clone)]
pub struct Worker {
    server: ServerHandle,
    finished_tx_chan: Receiver<SignedTransaction>,
    mempool: Arc<Mutex<Mempool>>,
}

impl Worker {
    pub fn new(
        server: &ServerHandle,
        finished_tx_chan: Receiver<SignedTransaction>,
        mempool: &Arc<Mutex<Mempool>>,
    ) -> Self {
        Self {
            server: server.clone(),
            finished_tx_chan,
            mempool: Arc::clone(mempool),
        }
    }

    pub fn start(self) {
        thread::Builder::new()
            .name("generator-worker".to_string())
            .spawn(move || {
                self.worker_loop();
            })
            .unwrap();
        info!("Generator initialized into paused mode");
    }

    fn worker_loop(&self) {
        loop {
            let _tx = self.finished_tx_chan.recv().expect("Receive finished tx error");
            // print!("worker recieved tx");
            let new_tx = _tx.clone();
            // TODO for student: insert this finished block to blockchain, and broadcast this block hash
            {
                self.mempool.lock().unwrap().insert(&_tx);
            }
            
            let mut tx_vec: Vec<H256> = Vec::new();
            tx_vec.push(new_tx.hash());
            
            self.server.broadcast(Message::NewTransactionHashes(tx_vec));
            
        }
    }
}
