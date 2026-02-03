use sea_orm::DatabaseConnection;
use alloy::{network::EthereumWallet, providers::{fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller}, Identity, RootProvider}};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DbConnection(pub DatabaseConnection);


#[derive(Debug, Clone)]
pub struct ProviderConnection(pub FillProvider<JoinFill<JoinFill<JoinFill<Identity, JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>>, NonceFiller>, WalletFiller<EthereumWallet>>, RootProvider>);



