use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct User { pub id: u32, pub username: String, pub password: String }

#[derive(Deserialize)] pub struct SignupInput { pub username: String, pub password: String }
#[derive(Deserialize)] pub struct SigninInput { pub username: String, pub password: String }
#[derive(Serialize)] pub struct SignupResponse { pub message: String }
#[derive(Serialize)] pub struct SigninResponse { pub token: String }
#[derive(Debug, Serialize, Deserialize)] pub struct Claims { pub sub: u32, pub exp: usize }
#[derive(Deserialize)] pub struct AmountRequest { pub qty: f64 }
#[derive(Deserialize)] pub struct OrderRequest { pub side: String, pub qty: f64, pub price: f64, pub asset: String }
#[derive(Deserialize)] pub struct CancelRequest { pub order_id: u64 }
#[derive(Serialize, Clone, Copy)] pub struct Balance { pub available: f64, pub locked: f64 }
#[derive(Serialize)] #[serde(rename_all = "camelCase")]
pub struct BalanceResponse { pub usd_balance: Balance, pub stock_balances: HashMap<String, Balance> }
