
use std::collections::HashMap;

use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use crate::config::config::AppConfig;
use crate::state_models::models::ProviderConnection;
use crate::error::error::AppError;
use once_cell::sync::Lazy;





// #[allow(unused)]
// pub static CHAINID_MAP : Lazy<HashMap<String , u32>> = Lazy::new(|| {
//     let mut map = HashMap::new();
//     map.insert("arbitrum_sepolia".to_string(), 421614);
//     map.insert("bitlayer_testnet".to_string(), 200810);
//     map.insert("base_sepolia".to_string(), 84532);
//     map.insert("base_mainnet".to_string(), 8453);
//     map.insert("umi_devnet".to_string(), 42069);
//     map
// });




pub static CHAIN_RPC : Lazy<HashMap<String , Vec< &'static str>>>= Lazy ::new(||{
    let mut map = HashMap::new();

    map.insert("bitlayer_testnet".to_string() , vec!["https://testnet-rpc.bitlayer.org"]);
    map.insert("base_sepolia".to_string(), vec!["https://rpc.ankr.com/base_sepolia/85681bbbd3308b54e7e23fa5a69a7be9dd7fe805e966f342aab236b8538d2017"]);
    map.insert("base_mainnet".to_string(), vec!["https://base-mainnet.g.alchemy.com/v2/je8NBeGlxHuC1m6VCHB93"]);
    map.insert("test_base_mainnet".to_string(), vec!["https://go.getblock.asia/399e2b8bd3fa44f3a9d05d3390ae43e2"]);
    map.insert("umi_devnet".to_string(), vec!["https://devnet.uminetwork.com/evm"]);
    map
});


pub async fn create_provider(chain: &String , worker_id: u64) -> Result<ProviderConnection, AppError> {

    let config = AppConfig::from_env()?;
    let signer;

    if worker_id == 0 {
        signer = &config.signer0;

    }else if worker_id==1{
        signer = &config.signer1;

    }else if worker_id==2{
        signer = &config.signer2;
        
    }else {
        signer = &config.signer3;
    }
    

    let rpc_list = CHAIN_RPC
        .get(chain)
        .ok_or_else(|| AppError::InternalError(format!("No RPCs found for chain {}", chain)))?;

    let private_key: PrivateKeySigner = signer.parse().expect("Cannot parse signer");

    for rpc in rpc_list {

        if let Ok(rpc_url) = rpc.parse() {

            let provider = ProviderBuilder::new()
                .with_cached_nonce_management()
                .wallet(private_key.clone())
                .connect_http(rpc_url);

            match provider.get_chain_id().await {
                Ok(_) => {
                    // println!("✅ Connected to RPC: {}", rpc);
                    return Ok(ProviderConnection(provider));
                }
                Err(err) => {
                    eprintln!("⚠️ Failed to connect to RPC {}: {}", rpc, err);
                    continue;
                }
            }
        }
    }

    Err(AppError::InternalError("All RPC endpoints failed".to_string()))
}


