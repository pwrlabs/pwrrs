use std::time::Duration;
use log::error;
use crate::{
    RPC,
    rpc::types::{ProcessVidaTransactions, BlockSaver, VidaTransactionSubscription}
};
use tokio::runtime::Runtime;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

impl VidaTransactionSubscription {
    pub fn new(
        pwrrs: Arc<RPC>,
        vida_id: u64,
        starting_block: u64,
        handler: ProcessVidaTransactions,
        poll_interval: u64,
        block_saver: Option<Box<dyn BlockSaver>>,
    ) -> Self {
        Self {
            pwrrs,
            vida_id,
            starting_block,
            poll_interval,
            latest_checked_block: Arc::new(std::sync::atomic::AtomicU64::new(starting_block)),
            handler,
            block_saver,
            wants_to_pause: Arc::new(AtomicBool::new(false)),
            paused: Arc::new(AtomicBool::new(false)),
            stop: Arc::new(AtomicBool::new(false)),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            error!("VidaTransactionSubscription is already running");
            return;
        }
    
        self.running.store(true, Ordering::SeqCst);
        self.wants_to_pause.store(false, Ordering::SeqCst);
        self.paused.store(false, Ordering::SeqCst);
        self.stop.store(false, Ordering::SeqCst);
        self.latest_checked_block.store(self.starting_block - 1, Ordering::SeqCst);
    
        let pwrrs = Arc::clone(&self.pwrrs);
        let vida_id = self.vida_id;
        let poll_interval = self.poll_interval;
        let handler = self.handler;
        let block_saver = self.block_saver.take(); // Take ownership of the block saver
        let wants_to_pause = Arc::clone(&self.wants_to_pause);
        let paused = Arc::clone(&self.paused);
        let stop = Arc::clone(&self.stop);
        let running = Arc::clone(&self.running);
        let latest_checked_block = Arc::clone(&self.latest_checked_block);
        println!("latest_checked_block: {}", latest_checked_block.load(Ordering::SeqCst));
    
        thread::Builder::new()
            .name(format!("VidaTransactionSubscription:VIDA-ID-{}", vida_id))
            .spawn(move || {
                let rt = Runtime::new().expect("Failed to create runtime");
                rt.block_on(async {
                    while !stop.load(Ordering::SeqCst) {
                        if wants_to_pause.load(Ordering::SeqCst) {
                            if !paused.load(Ordering::SeqCst) {
                                paused.store(true, Ordering::SeqCst);
                            }
                            thread::sleep(Duration::from_millis(10));
                            continue;
                        } else {
                            if paused.load(Ordering::SeqCst) {
                                paused.store(false, Ordering::SeqCst);
                            }
                        }

                        match pwrrs.get_latest_block().await {
                            Ok(latest_block) => {
                                if latest_block == latest_checked_block.load(Ordering::SeqCst) {
                                    continue;
                                }

                                let mut max_block_to_check: u64 = latest_block;

                                if latest_block > latest_checked_block.load(Ordering::SeqCst) + 1000 {
                                    max_block_to_check = latest_checked_block.load(Ordering::SeqCst) + 1000;
                                }

                                match pwrrs.get_vida_data_transactions(
                                    latest_checked_block.load(Ordering::SeqCst) + 1,
                                    max_block_to_check,
                                    vida_id
                                ).await {
                                    Ok(transactions) => {
                                        for transaction in transactions {
                                            match std::panic::catch_unwind(|| handler(transaction)) {
                                                Ok(_) => {},
                                                Err(_) => {
                                                    error!("Failed to process VIDA transaction - handler panicked");
                                                }
                                            }
                                        }

                                        latest_checked_block.store(max_block_to_check, Ordering::SeqCst);

                                        // Call block saver if provided
                                        if let Some(ref saver) = block_saver {
                                            saver.save_block(latest_checked_block.load(Ordering::SeqCst)).await;
                                        }
                                    },
                                    Err(e) => {
                                        error!("Failed to fetch VIDA transactions: {:?}", e);
                                    }
                                }
                            
                            },
                            Err(e) => {
                                error!("Failed to get latest block number: {:?}", e);
                            }
                        }
                        thread::sleep(Duration::from_millis(poll_interval));
                    }
                    running.store(false, Ordering::SeqCst);
                });
            })
            .expect("Failed to spawn thread");
    }

    pub fn pause(&self) {
        self.wants_to_pause.store(true, Ordering::SeqCst);
        
        // Block until actually paused
        while !self.paused.load(Ordering::SeqCst) && self.running.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn resume(&self) {
        self.wants_to_pause.store(false, Ordering::SeqCst);
    }

    pub fn stop(&self) {
        self.pause();
        self.stop.store(true, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn is_paused(&self) -> bool {
        self.wants_to_pause.load(Ordering::SeqCst)
    }

    pub fn is_stopped(&self) -> bool {
        self.stop.load(Ordering::SeqCst)
    }

    pub fn get_latest_checked_block(&self) -> u64 {
        self.latest_checked_block.load(Ordering::SeqCst)
    }

    pub fn set_latest_checked_block(&self, block_number: u64) {
        self.latest_checked_block.store(block_number, Ordering::SeqCst);
    }

    pub fn get_starting_block(&self) -> u64 {
        self.starting_block
    }

    pub fn get_vida_id(&self) -> u64 {
        self.vida_id
    }

    pub fn get_handler(&self) -> ProcessVidaTransactions {
        self.handler
    }

    pub fn get_pwrrs(&self) -> Arc<RPC> {
        Arc::clone(&self.pwrrs)
    }
}
