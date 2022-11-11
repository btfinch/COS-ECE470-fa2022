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
use crate::types::transaction::{generate_random_transaction_1, sign};

use std::sync::{Arc, Mutex};
use crate::blockchain::Blockchain;
use crate::types::hash::{H256, Hashable};

use crate::types::block::{generate_block};


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
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the generator thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(mempool: &Arc<Mutex<Mempool>>) -> (Context, Handle, Receiver<SignedTransaction>) { // should blockchain and mp have & infront here?
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    let (finished_tx_sender, finished_tx_receiver) = unbounded();
    let mempool_clone = Arc::clone(mempool);
    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        finished_tx_chan: finished_tx_sender,
        mempool: mempool_clone,
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle, finished_tx_receiver)
}

#[cfg(any(test,test_utilities))]
fn test_new() -> (Context, Handle, Receiver<SignedTransaction>) {
    let new_mp = Mempool::new();
    let wrapped_mp = Arc::new(Mutex::new(new_mp));
    new(&wrapped_mp)
}

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

            // Generating a random signed transaction
            let rand_signed_tx = generate_random_transaction_1();
            self.finished_tx_chan.send(rand_signed_tx.clone()).expect("Send finished tx error");
            {
                self.mempool.lock().unwrap().insert(&rand_signed_tx);
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