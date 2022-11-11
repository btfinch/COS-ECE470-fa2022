pub mod worker;

use log::info;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::mem;
use std::time;

use std::thread;

use crate::blockchain::Mempool;
use crate::types::block;
use crate::types::block::Block;
use crate::types::transaction::SignedTransaction;

use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use crate::types::hash::{H256, Hashable};

use crate::types::block::{generate_block};


enum ControlSignal {
    Start(u64), // the number controls the lambda of interval between block generation
    Update, // update the block in mining, it may due to new blockchain tip or new transaction
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
    finished_block_chan: Sender<Block>,
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Mempool>>,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(blockchain: &Arc<Mutex<Blockchain>>, mempool: &Arc<Mutex<Mempool>>) -> (Context, Handle, Receiver<Block>) { // should blockchain and mp have & infront here?
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    let (finished_block_sender, finished_block_receiver) = unbounded();
    let blockchain_clone = Arc::clone(blockchain); // note arc::clone is just creating another reference to same thing
    let mempool_clone = Arc::clone(mempool);
    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        finished_block_chan: finished_block_sender,
        blockchain: blockchain_clone, // am I allowed to have two variables with the same name like this?
        mempool: mempool_clone,
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle, finished_block_receiver)
}

#[cfg(any(test,test_utilities))]
fn test_new() -> (Context, Handle, Receiver<Block>) {
    let new_bc = Blockchain::new();
    let new_mp = Mempool::new();
    let wrapped_bc = Arc::new(Mutex::new(new_bc));
    let wrapped_mp = Arc::new(Mutex::new(new_mp));
    new(&wrapped_bc,&wrapped_mp)
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, lambda: u64) {
        self.control_chan
            .send(ControlSignal::Start(lambda))
            .unwrap();
    }

    pub fn update(&self) {
        self.control_chan.send(ControlSignal::Update).unwrap();
    }
}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner".to_string())
            .spawn(move || {
                self.miner_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn miner_loop(&mut self) {

        // let mut c_blockchain = Arc::clone(&self.blockchain); 
        let mut parent = self.blockchain.lock().unwrap().tip();
        // main mining loop

        // ***** Creating initial vec of transactions w block parameters to mine *****
        
        let max_transaction_count = 30; // number of transactions per block:
        let mut cont = 0;
        let mut block_transactions: Vec<SignedTransaction> = Vec::new();
        let mut keys_to_remove: Vec<H256> = Vec::new();

        

        let mut flag = 0;


        loop {
            if flag <=1 {
                // below might have lock for too long!
                {
                    cont =0;
                    for (key, value) in self.mempool.lock().unwrap().map.clone().into_iter(){
                        keys_to_remove.push(key.clone());
                        cont = cont+1;
                        if cont >= max_transaction_count{
                            break;
                        }
                    }
                }

                // how many keys to remove do we have?
                println!("keys_to_remove_length!!! {:?}",keys_to_remove.len());


                for i in 0..keys_to_remove.len(){

                    block_transactions.push(self.mempool.lock().unwrap().map.remove(&keys_to_remove[i]).unwrap());
                }  
                keys_to_remove.clear();

                flag = flag+ 1;
            }
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    match signal {
                        ControlSignal::Exit => {
                            info!("Miner shutting down");
                            self.operating_state = OperatingState::ShutDown;
                        }
                        ControlSignal::Start(i) => {
                            info!("Miner starting in continuous mode with lambda {}", i);
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
                                info!("Miner shutting down");
                                self.operating_state = OperatingState::ShutDown;
                            }
                            ControlSignal::Start(i) => {
                                info!("Miner starting in continuous mode with lambda {}", i);
                                self.operating_state = OperatingState::Run(i);
                            }
                            ControlSignal::Update => {
                                unimplemented!() // here I need to check what the new blockchain is and get the tip: How do I get the new blockchain?  
                            }
                        };
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Miner control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }

            // TODO for student: actual mining, create a block
            let mut dify: H256 = [255u8; 32].into();
            
            let c_parent = parent.clone();
            let c1_parent = parent.clone();
            // let d_blockchain = Arc::clone(&self.blockchain);
            match self.blockchain.lock().unwrap().map.get(&c_parent){
                Some(parent_block) => dify = parent_block.get_difficulty(), // May need to clone or dereference here
                _=> println!("Invalid Parent Hash"),
            }
            let c_dify = dify.clone();

            
            

            let new_block = generate_block(&c1_parent, &dify, &block_transactions); // Am I handling the merkle stuff right in new_block?
            let block = new_block.clone();
            
            // TODO for student: if block mining finished, you can have something like: self.finished_block_chan.send(block.clone()).expect("Send finished block error");
            if new_block.hash() <= c_dify {
                self.finished_block_chan.send(block.clone()).expect("Send finished block error");

                // this is duplicated is that a problem? (because miner worker also does this)
                {
                    self.blockchain.lock().unwrap().insert(&block);
                }
                println!("mined block");

                // Get new transactions to put in the block:
                block_transactions.clear();
                {
                    cont =0;
                    for (key, value) in self.mempool.lock().unwrap().map.clone().into_iter(){
                        keys_to_remove.push(key.clone());
                        cont = cont+1;
                        if cont >= max_transaction_count{
                            break;
                        }
                    }
                }
                
                for i in 0..keys_to_remove.len(){
                    block_transactions.push(self.mempool.lock().unwrap().map.remove(&keys_to_remove[i]).unwrap());
                }  
                keys_to_remove.clear();

                
            }
            {
                parent = self.blockchain.lock().unwrap().tip();
            }
            
            

            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

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

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST