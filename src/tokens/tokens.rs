use alloy::primitives::Address;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TokenError {
    #[error("Unknown chainId {0}")]
    UnknownChain(String),
    #[error("Unknown contract {0} for chainId {1}")]
    UnknownToken(String, String),
}



pub static TOKENS: Lazy<HashMap<String, HashMap<&'static str, Address>>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // let base_mainnet = HashMap::from([
    //     ("USDC", Address::from_str("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913").unwrap()),
    // ]);

    // let bnb_mainnet = HashMap::from([
    //     (
    //         "USDC",
    //         Address::from_str("0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d").unwrap(),
    //     ),
    //     (
    //         "USDT",
    //         Address::from_str("0x55d398326f99059fF775485246999027B3197955").unwrap(),
    //     )
    // ]);   

    let base_sepolia = HashMap::from([
        (
            "USDC",
            Address::from_str("0x6E5C7663971Be425B4726D7ba90456B935bb95ce").unwrap(),
        ),
        (
            "USDT",
            Address::from_str("0xB72FDb9f8190D8e1141e6a8e9c0732b0f4d93c09").unwrap(),
        )
    ]);   

    // let ethereum_mainnet = HashMap::from([
    //     (
    //         "USDC",
    //         Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(),
    //     ),
    //     (
    //         "USDT",
    //         Address::from_str("0xdAC17F958D2ee523a2206206994597C13D831ec7").unwrap(),
    //     )
    // ]);   


    // map.insert("bnb_mainnet".to_string(), bnb_mainnet);
    // map.insert("base_mainnet".to_string(), base_mainnet);
    map.insert("base_sepolia".to_string(), base_sepolia);
    // map.insert("ethereum_mainnet".to_string(), ethereum_mainnet);
    map
    
});

pub fn get_token(chain: String, token_name: &str) -> Result<Address, TokenError> {
    TOKENS
        .get(&chain)
        .ok_or(TokenError::UnknownChain(chain.clone()))?
        .get(token_name)
        .copied()
        .ok_or_else(|| TokenError::UnknownToken(token_name.to_string(), chain))
}
