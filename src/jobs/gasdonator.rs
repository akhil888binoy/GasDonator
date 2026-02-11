use std::{ str::FromStr, time::Duration};

use alloy::{
    network::TransactionBuilder, primitives::{ Address, U256, utils::parse_ether}, providers::Provider, rpc::types::TransactionRequest, sol
};
use rust_decimal::{Decimal};
use sea_orm::{
    ActiveModelTrait, ActiveValue::{ Set}, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait
};
use uuid::Uuid;

use tracing::{error, warn};
use tokio::time::sleep;
use crate::{
    chain_config::chain_config::{ create_provider}, config::config::AppConfig, entities::{ gas_donation, prelude::{ UserWallet},  user_wallet}, error::error::AppError, jobs::index::{MAX_RETRIES, RETRY_BACKOFF, between_cycles_cleanup}, state_models::models::DbConnection,  tokens::tokens::TOKENS,
};


sol!(
    #[sol(rpc)]
    ERC20,
    "src/utils/abi/ERC20.json"
);


// pub fn send_email_notification(
//     email_subject: String,
//     email_body: String,
// ) -> Result<(), AppError>{

//     let config = AppConfig::from_env().map_err(|e| AppError::InternalError(format!("Email config error: {e}")))?;

//     let email = Message::builder()
//             .from(format!("AvitusNotification <{}>", config.email_from ).parse().unwrap())
//             // .to("Harish <Harish@avitus.bet>".parse().unwrap())
//             // .cc("Aman <aman@avituslabs.xyz>".parse().unwrap())
//             .to("Akhil <akhil888binoy@gmail.com>".parse().unwrap())
//             // .cc("Arafath <arafathshariff21@gmail.com>".parse().unwrap())
//             .cc("Abin <abin@bitqcode.com>".parse().unwrap())
//             .subject(email_subject)
//             .header(ContentType::TEXT_PLAIN)
//             .body(email_body)
//             .unwrap();

//         let creds = Credentials::new(
//                 config.email_from.to_owned(),
//                 config.email_pass.to_owned(), 
//         );

//         let mailer = SmtpTransport::relay("smtp.gmail.com")
//         .map_err(|e| AppError::InternalError(format!("SMTP relay error: {e}")))?
//         .credentials(creds)
//         .build();

//         mailer.send(&email).map_err(|e| AppError::InternalError(format!("Email send failed: {e}")))?;

//         Ok(())
// }


pub async fn donate_wallet(
    worker_id: u64,
    db: &DbConnection,
) -> Result<(), AppError> {


            let txn = db.0.begin().await.map_err(AppError::DbError)?;

            // Select up to 100 pending requests and lock them for this worker
            let user_wallets = UserWallet::find()
                .filter(user_wallet::Column::Status.eq("FREE"))
                .filter(user_wallet::Column::ActiveGas.lt(0.1))
                .order_by_asc(user_wallet::Column::CreatedAt)
                .limit(100)
                .lock_exclusive() // <- row-level exclusive lock
                .all(&txn)
                .await
                .map_err(AppError::DbError)?;

            // Mark them as IN_PROGRESS so other workers skip them
            for req in &user_wallets {
                let mut active: user_wallet::ActiveModel = req.clone().into();
                active.status = Set("IN_PROGRESS".to_string());
                active.update(&txn).await.map_err(AppError::DbError)?;
            }

            // Commit so locks are released and status is updated
            txn.commit().await.map_err(AppError::DbError)?;

    for user_wallet in user_wallets{

        let mut retries = 0;

        loop {
            let result = process_single_request(worker_id, user_wallet.id, &db).await;

            match result {
            Ok(_) => {
                // success, move to next request
                break;
            },

            Err(e) if retries < MAX_RETRIES => {
                retries += 1;
                warn!("Retry {} for request {}: {}", retries, user_wallet.id, e);
                sleep(RETRY_BACKOFF * retries).await; // backoff
            },

            Err(e) => {
                    //  Reset back to PENDING for retry in next cycle

                    error!("Failed to process request {} after {} retries: {}", user_wallet.id, retries, e);

                    let mut complete_request: user_wallet::ActiveModel = user_wallet.clone().into();
                        complete_request.status = Set("FREE".to_string());
                        complete_request
                        .update(&db.0)
                        .await
                        .map_err(AppError::DbError)?;
                        break;
                
            }
        }
            sleep(Duration::from_millis(150)).await;
            between_cycles_cleanup(&db).await;
            tokio::task::yield_now().await;
        }
    
    
    
    }
    Ok(())
}



async fn process_single_request(
    worker_id: u64,
    user_wallet_id: Uuid,
    db: &DbConnection,
)->Result<(), AppError> {

    let config = AppConfig::from_env()?;
    let mut is_sweepable =false;
    let txn = db.0.begin().await.map_err(AppError::DbError)?;

    let master_wallet_address = Address::from_str(config.master_wallet_address.as_str()).unwrap() ;
    let pending_wallet = UserWallet::find_by_id(user_wallet_id)
    .lock_exclusive()
    .one(&txn)
    .await?;

    if pending_wallet.is_none() {
        return Ok(()); // already deleted or processed
    }

    let pending_wallet = pending_wallet.unwrap();
    println!("Wallet : {} in Worker : {}", pending_wallet.wallet_address, worker_id);
    let wallet_address: Address = pending_wallet.wallet_address.parse().map_err(|e|{
        eprintln!("Invalid wallet address {:?}",e);
        AppError::BadRequest("Invalid wallet address".to_string())
    } )?;

    let user_id = pending_wallet.user_id;


    for (chain_name , tokens) in TOKENS.iter(){
        println!("Checking Chain {} on Wallet {}", chain_name, wallet_address);
        let mut usdc_token_balance = U256::ZERO ;
        let mut usdt_token_balance= U256::ZERO;
        let mut total_gas_units = 0;

        let provider = create_provider(&chain_name, worker_id).await.map_err(|e| {
            eprintln!("Cannot create provider on  {:?}: {:?}", chain_name, e);
            AppError::InternalError(format!("Provider error: {e}"))
        })?;

        let  gas_balance  = provider.0.get_balance(wallet_address).await.map_err(|e| AppError::InternalError(format!("Cannot fetch native balance: {e}")))?;
    
        for (token_name , token_address) in tokens {

            let erc20 = ERC20::new(*token_address, &provider.0);

            if token_name ==  &"USDC" {

                usdc_token_balance  = erc20.balanceOf(wallet_address).call().await.map_err(|e|{   
                        eprintln!("Error fetching USDC balance for {:?}: {:?}", wallet_address, e);
                        AppError::InternalError(format!("Provider error: {e}"))
                    })?;
                
                if usdc_token_balance > U256::ZERO {

                    let call = erc20.transfer(master_wallet_address, usdc_token_balance);

                    let usdc_transfer_gas = call.from(wallet_address)
                        .estimate_gas()
                        .await.map_err(|e|{
                        eprintln!("Error Cannot estimate gas {:?}: {:?}", wallet_address, e);
                        AppError::InternalError(format!("Error Cannot estimate gas : {e}"))
                    } )?;

                    total_gas_units += usdc_transfer_gas;
                }

                // println!("Checking balance of  token {} on Wallet {} = {}", token_name, wallet_address, usdc_token_balance);
            
            }else {

                usdt_token_balance  = erc20.balanceOf(wallet_address).call().await.map_err(|e|{
                    eprintln!("Error fetching USDT balance for {:?}: {:?}", wallet_address, e);
                    AppError::InternalError(format!("Provider error: {e}"))
                } )?;

            if usdt_token_balance > U256::ZERO {
                let call = erc20.transfer(master_wallet_address, usdt_token_balance);

                let usdt_transfer_gas = call.from(wallet_address)
                    .estimate_gas()
                    .await.map_err(|e|{
                    eprintln!("Error Cannot estimate gas {:?}: {:?}", wallet_address, e);
                    AppError::InternalError(format!("Error Cannot estimate gas : {e}"))
                } )?;
                    total_gas_units += usdt_transfer_gas;
                }
            }
            
        }

        let gas_price = provider.0.get_gas_price().await.map_err(|e|{
                    eprintln!("Error Cannot get gas price {:?}: {:?}", wallet_address, e);
                    AppError::InternalError(format!("Error Cannot get gas price : {e}"))
            } )?;

        let total_gas_units_u256 = U256::from(total_gas_units);
        let mut  minimum_gas = total_gas_units_u256 * U256::from(gas_price);

        // +20% buffer
        // let buffer = minimum_gas / U256::from(5);

        minimum_gas = minimum_gas * U256::from(2);


        let deficit = minimum_gas.saturating_sub(gas_balance);
        
        println!("Total Gas Units :{}", total_gas_units);
        println!("Gas Price :{}", gas_price);
        println!("Minimum Gas Needed {}", minimum_gas );
        println!("Deficit Gas Sent {}", deficit);

        if usdc_token_balance > U256::from(0) || usdt_token_balance > U256::from(0) {

                        
                if  gas_balance <= minimum_gas  {

                                    println!("Eligible for gas Wallet : {}", wallet_address);

                                    let min_topup = parse_ether("0.00000001").unwrap();

                                    if deficit >= min_topup {

                                        println!("Passed Dust Condition {}", wallet_address);
                                        let tx = TransactionRequest::default().with_to(wallet_address).with_value(deficit);

                                        let pending = provider.0.send_transaction(tx).await.map_err(|e| {
                                            eprintln!("Failed to send gas {:?}: {:?}", wallet_address, e);
                                            AppError::InternalError(format!("Provider error: {e}"))
                                        })?;
                                        
                                        println!("Transaction Hash {}", pending.tx_hash());
                                        
                                        is_sweepable = true;

                                        gas_donation::ActiveModel{
                                            id: Set(Uuid::new_v4()),
                                            user_id: Set(user_id),
                                            wallet_address: Set(wallet_address.to_string()),
                                            chain : Set(chain_name.clone()),
                                            gas: Set(Decimal::from_str(&deficit.to_string()).unwrap()),
                                            created_at: Set(chrono::Utc::now().into()),
                                        }
                                        .insert(&txn)
                                                .await
                                                .map_err(|e|{
                                                    eprintln!("Failed to inser to DB {:?}", e);
                                                    AppError::DbError(e)
                                        })?;
                        }else {
                        is_sweepable = true;
                    }
            }else{
                println!("No More Gas Needed  for Wallet : {} on chain {}", wallet_address, chain_name );
                is_sweepable = true;
            }
        }else{
            println!("No Balance  in  Wallet : {} on chain {}", wallet_address, chain_name );
        }

    }

    println!("Is Sweepable  {}", is_sweepable);

    if is_sweepable {
        let mut complete_request: user_wallet::ActiveModel = pending_wallet.into();
                        complete_request.status = Set("SWEEPABLE".to_string());
                        complete_request
                        .update(&txn)
                        .await
                        .map_err(AppError::DbError)?;
    }else {
        let mut complete_request: user_wallet::ActiveModel = pending_wallet.into();
                        complete_request.status = Set("FREE".to_string());
                        complete_request
                        .update(&txn)
                        .await
                        .map_err(AppError::DbError)?;
    }

    txn.commit().await.map_err(AppError::DbError)?;

    // if  withdraw_alert_10 {
    //     if let Err(e) = send_email_notification("⚠️ User Withdrawal 10% of Pool in 24h".to_string(), format!("User Withdrawing 10% of Pool in 24h : userId {} ", user_id).to_string()){
    //             error!("Email alert failed: {:?}", e);
    //     };
    // }

    // if  withdraw_alert {
    //     if let Err(e) = send_email_notification("⚠️ User Withdrawal 5% of Pool".to_string(), format!("User Withdrawing 5% of Pool : userId {} ", user_id).to_string()){
    //         error!("Email alert failed: {:?}", e);
    //     };
    // }

    // if should_alert {
    //     if let Err(e) = send_email_notification("⚠️ Vault Liquidity Alert".to_string(), "Liquidity Limit reached".to_string()){
    //         error!("Email alert failed: {:?}", e);
    //     };
    // }

    Ok(())
}

