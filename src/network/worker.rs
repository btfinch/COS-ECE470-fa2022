use super::message::Message;
use super::peer;
use super::server::Handle as ServerHandle;
use crate::types::hash::{H256, Hashable};

use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use crate::types::block::Block;

use log::{debug, warn, error};

use std::thread;

#[cfg(any(test,test_utilities))]
use super::peer::TestReceiver as PeerTestReceiver;
#[cfg(any(test,test_utilities))]
use super::server::TestReceiver as ServerTestReceiver;
#[derive(Clone)]
pub struct Worker {
    msg_chan: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
}


impl Worker {
    pub fn new(
        num_worker: usize,
        msg_src: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
        server: &ServerHandle,
        blockchain: &Arc<Mutex<Blockchain>>, // should this be a reference?
    ) -> Self {
        Self {
            msg_chan: msg_src,
            num_worker,
            server: server.clone(),
            blockchain: Arc::clone(blockchain),
        }
    }

    pub fn start(self) {
        let num_worker = self.num_worker;
        for i in 0..num_worker {
            let cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }

    fn worker_loop(&self) {
        loop {
            let result = smol::block_on(self.msg_chan.recv());
            if let Err(e) = result {
                error!("network worker terminated {}", e);
                break;
            }
            let msg = result.unwrap();
            let (msg, mut peer) = msg;
            let msg: Message = bincode::deserialize(&msg).unwrap();
            match msg {
                Message::Ping(nonce) => {
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }
                Message::NewBlockHashes(nonce) => {
                    let mut not_contained: Vec<H256> = Vec::new();
                    for i in 0..nonce.len(){
                        if !self.blockchain.lock().unwrap().map.contains_key(&nonce[i]){
                            not_contained.push(nonce[i].clone());
                        }
                    }
                    if !not_contained.clone().is_empty(){
                        peer.write(Message::GetBlocks(not_contained));
                    }

                    println!("NewBlockHashes recieved");
                     // should this be peer.write or server.broadcast
                }
                Message::GetBlocks(nonce) =>{
                    let mut contained: Vec<Block> = Vec::new();
                    for i in 0..nonce.len(){

                        {
                            if self.blockchain.lock().unwrap().map.contains_key(&nonce[i]){
                                contained.push(self.blockchain.lock().unwrap().map[&nonce[i]].clone());
                            }
                            else {
                                // println!("not contained in blockchain");
                                // print!("{:?}",self.blockchain.lock().unwrap().all_blocks_in_longest_chain());
                            }
                        }
                        /* 
                        match self.blockchain.lock().unwrap().map.get(&nonce[i]){
                            Some(block) => contained.push(block.clone()), // May need to clone or dereference here
                            _=> println!("requested block not in blockchain"),
                            
                        }
                        */
                    }
                    if !contained.clone().is_empty(){
                        let cc = contained.clone();
                        peer.write(Message::Blocks(contained));
                        // println!("sent length {:?}",cc.len());
                    }
                    // println!("GetBlocks Request recieved");
                    
                }
                Message::Blocks(nonce) =>{
                    let mut new_blocks: Vec<H256> = Vec::new();
                    for i in 0..nonce.len(){
                        let hash = nonce[i].clone().hash();
                        {
                            let mut b_chain = self.blockchain.lock().unwrap();
                            if !b_chain.map.contains_key(&hash){
                                b_chain.insert(&nonce[i]);
                                /* insert will print invalid parent hash if parent is not 
                                contained in the blockchain, as this is right now blocks 
                                need to come in the right order and have valid parent hashes
                                */
                                new_blocks.push(hash);

                            }

                        }
                        
                        
                    }
                    // print tip

                    // println!("blocks recieved");
                    {
                        println!("{:?}",self.blockchain.lock().unwrap().tip());
                    }
                    if !new_blocks.clone().is_empty(){
                        self.server.broadcast(Message::NewBlockHashes(new_blocks));
                    }
                    
                }
                _ => unimplemented!(),
            }
        }
    }
}

#[cfg(any(test,test_utilities))]
struct TestMsgSender {
    s: smol::channel::Sender<(Vec<u8>, peer::Handle)>
}
#[cfg(any(test,test_utilities))]
impl TestMsgSender {
    fn new() -> (TestMsgSender, smol::channel::Receiver<(Vec<u8>, peer::Handle)>) {
        let (s,r) = smol::channel::unbounded();
        (TestMsgSender {s}, r)
    }

    fn send(&self, msg: Message) -> PeerTestReceiver {
        let bytes = bincode::serialize(&msg).unwrap();
        let (handle, r) = peer::Handle::test_handle();
        smol::block_on(self.s.send((bytes, handle))).unwrap();
        r
    }
}
#[cfg(any(test,test_utilities))]
/// returns two structs used by tests, and an ordered vector of hashes of all blocks in the blockchain
fn generate_test_worker_and_start() -> (TestMsgSender, ServerTestReceiver, Vec<H256>) {
    let (server, server_receiver) = ServerHandle::new_for_test();
    let (test_msg_sender, msg_chan) = TestMsgSender::new();
    let new_bc = Blockchain::new();
    let wrapped_bc = Arc::new(Mutex::new(new_bc));
    let worker = Worker::new(1, msg_chan, &server, &wrapped_bc);
    let mut longest_chain_hashes: Vec<H256> = Vec::new();
    {
        longest_chain_hashes = wrapped_bc.lock().unwrap().all_blocks_in_longest_chain(); // probably could subsitute this with the genesis block hash directly
    }

    worker.start(); 
    (test_msg_sender, server_receiver, longest_chain_hashes)
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use ntest::timeout;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;

    use super::super::message::Message;
    use super::generate_test_worker_and_start;

    #[test]
    #[timeout(60000)]
    fn reply_new_block_hashes() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut peer_receiver = test_msg_sender.send(Message::NewBlockHashes(vec![random_block.hash()]));
        let reply = peer_receiver.recv();
        if let Message::GetBlocks(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_get_blocks() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let h = v.last().unwrap().clone();
        let mut peer_receiver = test_msg_sender.send(Message::GetBlocks(vec![h.clone()]));
        let reply = peer_receiver.recv();
        if let Message::Blocks(v) = reply {
            assert_eq!(1, v.len());
            assert_eq!(h, v[0].hash())
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_blocks() {
        let (test_msg_sender, server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut _peer_receiver = test_msg_sender.send(Message::Blocks(vec![random_block.clone()]));
        let reply = server_receiver.recv().unwrap();
        if let Message::NewBlockHashes(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST