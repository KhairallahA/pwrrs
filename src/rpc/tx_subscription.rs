use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use log::{error};
use crate::{RPC, transaction::types::VMDataTransaction};
use tokio::runtime::Runtime;

pub struct IvaTransactionSubscription {
    pwrrs: Arc<RPC>,
    vm_id: u64,
    starting_block: u64,
    latest_checked_block: u64,
    handler: Arc<dyn IvaTransactionHandler>,
    pause: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
}

pub trait IvaTransactionHandler: Send + Sync {
    fn process_iva_transactions(&self, transaction: VMDataTransaction);
}

impl IvaTransactionSubscription {
    pub fn new(
        pwrrs: Arc<RPC>,
        vm_id: u64,
        starting_block: u64,
        handler: Arc<dyn IvaTransactionHandler>,
        _poll_interval: u64,
    ) -> Self {
        Self {
            pwrrs,
            vm_id,
            starting_block,
            latest_checked_block: 0,
            handler,
            pause: Arc::new(AtomicBool::new(false)),
            stop: Arc::new(AtomicBool::new(false)),
            running: Arc::new(AtomicBool::new(false)),
            thread_handle: None,
        }
    }

    pub fn start(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            error!("IvaTransactionSubscription is already running");
            return;
        }
    
        self.running.store(true, Ordering::SeqCst);
        self.pause.store(false, Ordering::SeqCst);
        self.stop.store(false, Ordering::SeqCst);
    
        let mut current_block = self.starting_block;
        let pwrrs = Arc::clone(&self.pwrrs);
        let vm_id = self.vm_id;
        let handler = Arc::clone(&self.handler);
        let pause = Arc::clone(&self.pause);
        let stop = Arc::clone(&self.stop);
        let running = Arc::clone(&self.running);
    
        let thread = thread::Builder::new()
            .name(format!("IvaTransactionSubscription:IVA-ID-{}", vm_id))
            .spawn(move || {
                let rt = Runtime::new().expect("Failed to create runtime");
                rt.block_on(async {
                    while !stop.load(Ordering::SeqCst) {
                        if pause.load(Ordering::SeqCst) {
                            continue;
                        }
    
                        if let Ok(latest_block) = pwrrs.get_latest_block_number().await {
                            let effective_latest_block = if latest_block > current_block + 1000 {
                                current_block + 1000
                            } else {
                                latest_block
                            };
    
                            if effective_latest_block >= current_block {
                                if let Ok(transactions) = pwrrs.get_vm_data_transactions(current_block, effective_latest_block, vm_id).await {
                                    for transaction in transactions {
                                        handler.process_iva_transactions(transaction);
                                    }
                                    current_block = effective_latest_block + 1;
                                }
                            }
                        }
                        thread::sleep(Duration::from_millis(100));
                    }
                    running.store(false, Ordering::SeqCst);
                });
            })
            .expect("Failed to spawn thread");
    
        self.thread_handle = Some(thread);
    }

    pub fn pause(&self) {
        self.pause.store(true, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        self.pause.store(false, Ordering::SeqCst);
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
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
        self.latest_checked_block
    }

    pub fn get_starting_block(&self) -> u64 {
        self.starting_block
    }

    pub fn get_vm_id(&self) -> u64 {
        self.vm_id
    }

    pub fn get_handler(&self) -> Arc<dyn IvaTransactionHandler> {
        Arc::clone(&self.handler)
    }
}
