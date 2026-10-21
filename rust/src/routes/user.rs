use std::collections::HashMap;
use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use crate::{middleware::AuthUser, orderbook::{Event, Order}, types::user::{AmountRequest, Balance, BalanceResponse, CancelRequest, Claims, OrderRequest, SigninInput, SigninResponse, SignupInput, SignupResponse, User}, AppState};

fn error(status: actix_web::http::StatusCode, message: &str) -> HttpResponse { HttpResponse::build(status).json(serde_json::json!({ "message": message })) }
fn valid_amount(value: f64) -> bool { value.is_finite() && value > 0.0 }
fn get_balance(map: &mut HashMap<u32, Balance>, user: u32) -> &mut Balance { map.entry(user).or_insert(Balance { available: 0.0, locked: 0.0 }) }

#[post("/signup")]
pub async fn signup(body: web::Json<SignupInput>, state: web::Data<AppState>) -> impl Responder {
    if body.username.trim().len() < 3 || body.password.len() < 6 { return error(actix_web::http::StatusCode::BAD_REQUEST, "Username must have 3 characters and password must have 6 characters"); }
    let mut users = state.users.lock().unwrap();
    if users.iter().any(|u| u.username == body.username.trim()) { return error(actix_web::http::StatusCode::CONFLICT, "User already exists"); }
    let mut id = state.next_user_id.lock().unwrap(); let user_id = *id; *id += 1;
    users.push(User { id: user_id, username: body.username.trim().to_string(), password: body.password.clone() });
    get_balance(&mut state.usd_balances.lock().unwrap(), user_id);
    state.stock_balances.lock().unwrap().insert(user_id, HashMap::new());
    HttpResponse::Created().json(SignupResponse { message: "Successfully signed up".into() })
}

#[post("/signin")]
pub async fn signin(body: web::Json<SigninInput>, state: web::Data<AppState>) -> impl Responder {
    let users = state.users.lock().unwrap();
    let Some(user) = users.iter().find(|u| u.username == body.username && u.password == body.password) else { return error(actix_web::http::StatusCode::UNAUTHORIZED, "Incorrect credentials"); };
    let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;
    let token = encode(&Header::default(), &Claims { sub: user.id, exp }, &EncodingKey::from_secret(crate::middleware::JWT_SECRET.as_bytes())).unwrap();
    HttpResponse::Ok().json(SigninResponse { token })
}

#[get("/balance")]
pub async fn balance_route(user: AuthUser, state: web::Data<AppState>) -> impl Responder {
    let usd = get_balance(&mut state.usd_balances.lock().unwrap(), user.0).to_owned();
    let stocks = state.stock_balances.lock().unwrap().get(&user.0).cloned().unwrap_or_default();
    HttpResponse::Ok().json(BalanceResponse { usd_balance: usd, stock_balances: stocks })
}
pub use balance_route as balance;

#[post("/onramp")]
pub async fn onramp(user: AuthUser, body: web::Json<AmountRequest>, state: web::Data<AppState>) -> impl Responder {
    if !valid_amount(body.qty) { return error(actix_web::http::StatusCode::BAD_REQUEST, "Quantity must be positive"); }
    get_balance(&mut state.usd_balances.lock().unwrap(), user.0).available += body.qty;
    HttpResponse::Ok().json(serde_json::json!({ "message": "USD added" }))
}

#[post("/deposit/{asset}")]
pub async fn deposit(user: AuthUser, path: web::Path<String>, body: web::Json<AmountRequest>, state: web::Data<AppState>) -> impl Responder {
    if path.as_str() != "sol" || !valid_amount(body.qty) { return error(actix_web::http::StatusCode::BAD_REQUEST, "Only positive SOL deposits are supported"); }
    let mut stocks = state.stock_balances.lock().unwrap(); let sol = stocks.entry(user.0).or_default().entry("sol".into()).or_insert(Balance { available: 0.0, locked: 0.0 }); sol.available += body.qty;
    HttpResponse::Ok().json(serde_json::json!({ "message": "SOL deposited" }))
}

#[get("/orderbook")]
pub async fn orderbook(state: web::Data<AppState>) -> impl Responder { HttpResponse::Ok().json(state.orderbook.lock().unwrap().snapshot()) }

#[get("/orders")]
pub async fn orders(user: AuthUser, state: web::Data<AppState>) -> impl Responder { HttpResponse::Ok().json(state.orderbook.lock().unwrap().user_orders(user.0)) }

#[post("/order")]
pub async fn order(user: AuthUser, body: web::Json<OrderRequest>, state: web::Data<AppState>) -> impl Responder {
    if body.asset != "sol" || (body.side != "bid" && body.side != "ask") || !valid_amount(body.qty) || !valid_amount(body.price) { return error(actix_web::http::StatusCode::BAD_REQUEST, "A valid SOL side, price, and quantity are required"); }
    let mut usd = state.usd_balances.lock().unwrap(); let mut stocks = state.stock_balances.lock().unwrap();
    let usd_balance = get_balance(&mut usd, user.0).to_owned();
    let sol_available = stocks.get(&user.0).and_then(|assets| assets.get("sol")).map_or(0.0, |b| b.available);
    if body.side == "bid" && usd_balance.available < body.price * body.qty { return error(actix_web::http::StatusCode::UNPROCESSABLE_ENTITY, "Insufficient USD"); }
    if body.side == "ask" && sol_available < body.qty { return error(actix_web::http::StatusCode::UNPROCESSABLE_ENTITY, "Insufficient SOL"); }
    let events = state.orderbook.lock().unwrap().add(user.0, &body.side, body.price, body.qty);
    for event in &events { match event {
        Event::Resting { side, price, qty, .. } if side == "bid" => { let b = get_balance(&mut usd, user.0); b.available -= price * qty; b.locked += price * qty; },
        Event::Resting { side, qty, .. } if side == "ask" => { let sol = stocks.entry(user.0).or_default().entry("sol".into()).or_insert(Balance { available: 0.0, locked: 0.0 }); sol.available -= qty; sol.locked += qty; },
        Event::Fill(fill) => { let value = fill.price * fill.qty; get_balance(&mut usd, fill.buyer).locked -= value; get_balance(&mut usd, fill.seller).available += value; let buyer = stocks.entry(fill.buyer).or_default().entry("sol".into()).or_insert(Balance { available: 0.0, locked: 0.0 }); buyer.available += fill.qty; let seller = stocks.entry(fill.seller).or_default().entry("sol".into()).or_insert(Balance { available: 0.0, locked: 0.0 }); seller.locked -= fill.qty; },
        _ => {}
    } }
    HttpResponse::Created().json(serde_json::json!({ "events": events }))
}

#[post("/cancel")]
pub async fn cancel(user: AuthUser, body: web::Json<CancelRequest>, state: web::Data<AppState>) -> impl Responder {
    let Some(order) = state.orderbook.lock().unwrap().cancel(user.0, body.order_id) else { return error(actix_web::http::StatusCode::NOT_FOUND, "Open order not found"); };
    let remaining = order.qty - order.filled_qty; let mut usd = state.usd_balances.lock().unwrap(); let mut stocks = state.stock_balances.lock().unwrap();
    if order.side == "bid" { let b = get_balance(&mut usd, user.0); b.locked -= order.price * remaining; b.available += order.price * remaining; } else { let sol = stocks.entry(user.0).or_default().entry("sol".into()).or_insert(Balance { available: 0.0, locked: 0.0 }); sol.locked -= remaining; sol.available += remaining; }
    HttpResponse::Ok().json(serde_json::json!({ "message": "Order cancelled" }))
}
