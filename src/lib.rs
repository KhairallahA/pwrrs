//! PWR chain Rust SDK
//!
//! # Create wallet
//! ```ignore
//! # use pwr_rs::wallet::Wallet;
//! let wallet = Wallet::random();
//! ```
//!
//! # Create RPC
//! ```ignore
//! # use pwr_rs::wallet::Wallet;
//! # use pwr_rs::rpc::RPC;
//! # let wallet = Wallet::random();
//! let rpc = RPC::new("https://pwrrpc.pwrlabs.io/").unwrap();
//! let balance = rpc.balance_of_address(&wallet.address()).await.unwrap();
//! ```

pub mod block;
#[cfg(feature = "rpc")]
pub mod rpc;
pub mod wallet;

pub use wallet::{PublicKey, Wallet};
