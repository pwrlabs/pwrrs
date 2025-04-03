use std::time::Duration;
use log::error;
use crate::{
    RPC,
    rpc::types::{ProcessVidaTransactions, VidaTransactionSubscription}
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
        _poll_interval: u64,
    ) -> Self {
        Self {
            pwrrs,
            vida_id,
            starting_block,
            latest_checked_block: Arc::new(std::sync::atomic::AtomicU64::new(starting_block)),
            handler,
            pause: Arc::new(AtomicBool::new(false)),
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
        self.pause.store(false, Ordering::SeqCst);
        self.stop.store(false, Ordering::SeqCst);
    
        let pwrrs = Arc::clone(&self.pwrrs);
        let vida_id = self.vida_id;
        let handler = self.handler;
        let pause = Arc::clone(&self.pause);
        let stop = Arc::clone(&self.stop);
        let running = Arc::clone(&self.running);
        let latest_checked_block = Arc::clone(&self.latest_checked_block);
    
        let mut current_block = self.starting_block;

        thread::Builder::new()
            .name(format!("VidaTransactionSubscription:VIDA-ID-{}", vida_id))
            .spawn(move || {
                let rt = Runtime::new().expect("Failed to create runtime");
                rt.block_on(async {
                    while !stop.load(Ordering::SeqCst) {
                        if pause.load(Ordering::SeqCst) {
                            continue;
                        }

                        let latest_block = pwrrs.get_latest_block_number().await.unwrap();

                        let mut effective_latest_block = latest_block;
                        if latest_block > current_block + 1000 {
                            effective_latest_block = current_block + 1000;
                        }

                        if effective_latest_block >= current_block {
                            let transactions = pwrrs.get_vm_data_transactions(
                                current_block, effective_latest_block, vida_id
                            ).await.unwrap();

                            for transaction in transactions {
                                handler(transaction);
                            }

                            latest_checked_block.store(effective_latest_block, Ordering::SeqCst);
                            current_block = effective_latest_block + 1;
                        }
                        thread::sleep(Duration::from_millis(100));
                    }
                    running.store(false, Ordering::SeqCst);
                });
            })
            .expect("Failed to spawn thread");
    }

    pub fn pause(&self) {
        self.pause.store(true, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        self.pause.store(false, Ordering::SeqCst);
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn is_paused(&self) -> bool {
        self.pause.load(Ordering::SeqCst)
    }

    pub fn is_stopped(&self) -> bool {
        self.stop.load(Ordering::SeqCst)
    }

    pub fn get_latest_checked_block(&self) -> u64 {
        self.latest_checked_block.load(Ordering::SeqCst)
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
}
