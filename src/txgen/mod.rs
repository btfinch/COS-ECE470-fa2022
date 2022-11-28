pub mod worker;

use log::info;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use smol::net::unix::SocketAddr;
use std::mem;
use std::time;

use std::thread;

use crate::blockchain::Mempool;
use crate::types::block;
use crate::types::block::Block;
use crate::types::transaction::SignedTransaction;
use crate::types::transaction::Transaction;
use crate::types::transaction::{generate_random_transaction_1, sign};

use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use crate::types::hash::{H256, Hashable};

use crate::types::block::{generate_block};

use ring::signature::{Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters, self};
use std::net;
use crate::types::address::Address;
use rand::{thread_rng,Rng};


enum ControlSignal {
    Start(u64), // the number controls the theta of interval between tx generation
    Update, // update the tx 
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    finished_tx_chan: Sender<SignedTransaction>,
    mempool: Arc<Mutex<Mempool>>,
    controlled_nodes: Vec<Ed25519KeyPair>,
    all_adresses: Vec<Address>,
    blockchain: Arc<Mutex<Blockchain>>,
    // here we make transactions, we need everything that is going to go into the transaction here, even if we pass it on into the transaction crate to actually do the work
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the generator thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(mempool: &Arc<Mutex<Mempool>>, blockchain: &Arc<Mutex<Blockchain>>, p2p_address: net::SocketAddr) -> (Context, Handle, Receiver<SignedTransaction>) { // should blockchain and mp have & infront here?
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    let (finished_tx_sender, finished_tx_receiver) = unbounded();
    let mempool_clone = Arc::clone(mempool);
    let blockchain_clone = Arc::clone(blockchain);

    let mut all_addr: Vec<Address> = Vec::new();
    for i in 0..9{
        let kp_seed = &[i;32];
        let key_pair = Ed25519KeyPair::from_seed_unchecked(kp_seed).unwrap();
        let public_key_bytes = key_pair.public_key();
        let produced_public_key = public_key_bytes.clone().as_ref().to_vec();
        let generated = Address::from_public_key_bytes(public_key_bytes.as_ref());
        all_addr.push(generated);
    }

    let mut my_nodes: Vec<Ed25519KeyPair> = Vec::new();
    let mut node_num: u8;

    if p2p_address == "127.0.0.1:6000".parse::<net::SocketAddr>().unwrap() {
        
        for i in 0..3{
            let kp_seed = &[i;32];
            let key_pair = Ed25519KeyPair::from_seed_unchecked(kp_seed).unwrap();
            my_nodes.push(key_pair);
        }

    } else if p2p_address == "127.0.0.1:6001".parse::<net::SocketAddr>().unwrap(){
        for i in 3..6{
            let kp_seed = &[i;32];
            let key_pair = Ed25519KeyPair::from_seed_unchecked(kp_seed).unwrap();
            my_nodes.push(key_pair);
        }
    }else if p2p_address == "127.0.0.1:6002".parse::<net::SocketAddr>().unwrap(){
        for i in 6..9{
            let kp_seed = &[i;32];
            let key_pair = Ed25519KeyPair::from_seed_unchecked(kp_seed).unwrap();
            my_nodes.push(key_pair);
        }
    } 


    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        finished_tx_chan: finished_tx_sender,
        mempool: mempool_clone,
        controlled_nodes: my_nodes,
        all_adresses: all_addr,
        blockchain: blockchain_clone,

    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle, finished_tx_receiver)
}
/* 
#[cfg(any(test,test_utilities))]
fn test_new() -> (Context, Handle, Receiver<SignedTransaction>) {
    let new_mp = Mempool::new();
    let wrapped_mp = Arc::new(Mutex::new(new_mp));
    new(&wrapped_mp)
}
*/

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, theta: u64) {
        self.control_chan
            .send(ControlSignal::Start(theta))
            .unwrap();
    }

    pub fn update(&self) {
        self.control_chan.send(ControlSignal::Update).unwrap();
    }
}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("generator".to_string())
            .spawn(move || {
                self.generator_loop();
            })
            .unwrap();
        info!("Generator initialized into paused mode");
    }

    fn generator_loop(&mut self) {
        loop {
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    match signal {
                        ControlSignal::Exit => {
                            info!("Generator shutting down");
                            self.operating_state = OperatingState::ShutDown;
                        }
                        ControlSignal::Start(i) => {
                            info!("Generator starting in continuous mode with theta {}", i);
                            self.operating_state = OperatingState::Run(i);
                        }
                        ControlSignal::Update => {
                            // in paused state, don't need to update
                        }
                    };
                    continue;
                }
                OperatingState::ShutDown => {
                    return;
                }
                _ => match self.control_chan.try_recv() {
                    Ok(signal) => {
                        match signal {
                            ControlSignal::Exit => {
                                info!("Generator shutting down");
                                self.operating_state = OperatingState::ShutDown;
                            }
                            ControlSignal::Start(i) => {
                                info!("Generator starting in continuous mode with theta {}", i);
                                self.operating_state = OperatingState::Run(i);
                            }
                            ControlSignal::Update => {
                                unimplemented!() // here I need to check what the new blockchain is and get the tip: How do I get the new blockchain?  
                            }
                        };
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Generator control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }
            // Generating a random signed transaction for each controlled node
            for key_pair_index in 0..self.controlled_nodes.len(){
                let key_pair = self.controlled_nodes.get(key_pair_index).clone().unwrap();
                let public_key_bytes = key_pair.public_key();
                let produced_public_key = public_key_bytes.clone().as_ref().to_vec();
                let sender_address = Address::from_public_key_bytes(public_key_bytes.as_ref());
                let sender_address_1 = sender_address.clone();

                // getting value and account nonce in address
                let mut tip_hash: H256;

                {
                    tip_hash = self.blockchain.lock().unwrap().tip();
                }

                // first check if sender_addres is contained:

                let mut gogogo = 0;
                {
                    if self.blockchain.lock().unwrap().state_map.get(&tip_hash).expect("no state for tip_hash").contains_key(&sender_address_1){
                        gogogo = 1;
                    }
                }

                // then gogogo if flag is correct:

                if gogogo ==1{
                    let mut sender_nonce: u32;
                    let mut sender_balance: u32;
                    
                    {
                        (sender_nonce,sender_balance) = *self.blockchain.lock().unwrap().state_map.get(&tip_hash).expect("no state for tip_hash").get(&sender_address_1).expect("no values for address");
                    }
                    let go_nonce = sender_nonce +1;
                    let go_val = sender_balance/20;
                    // picking a random existing address to send it to
                    let mut rng = rand::thread_rng();
                    let rand_index: usize = rng.gen_range(0..9);
                    let r_addy = self.all_adresses[rand_index].clone();
                    // creating the signed transaction
                    let transact = Transaction::new(sender_address, r_addy, go_val, go_nonce);
                    let tx_c = transact.clone();
                    let signat = sign(&transact,&key_pair); // might need to clone here!!!
                    let signed_tx = SignedTransaction::new(tx_c, signat.as_ref().to_vec(), produced_public_key);

                    // sending the transaction and adding it to the mempool
                    self.finished_tx_chan.send(signed_tx.clone()).expect("Send finished tx error");
                    {
                        self.mempool.lock().unwrap().insert(&signed_tx);
                    }
                }
                

            }
            
            
            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64); // if code is not working try undoing this, made the sleep longer to make up for 9 transactions being processed at a time
                    thread::sleep(interval);
                }
            }
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST
/* 
#[cfg(test)]
mod test {
    use ntest::timeout;
    use crate::types::hash::Hashable;

    #[test]
    #[timeout(60000)]
    fn miner_three_block() {
        let (miner_ctx, miner_handle, finished_block_chan) = super::test_new();
        miner_ctx.start();
        miner_handle.start(0);
        let mut block_prev = finished_block_chan.recv().unwrap();
        for _ in 0..2 {
            let block_next = finished_block_chan.recv().unwrap();
            assert_eq!(block_prev.hash(), block_next.get_parent());
            block_prev = block_next;
        }
    }
}
*/

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST