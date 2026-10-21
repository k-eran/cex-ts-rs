use std::{collections::HashMap, sync::Mutex};
use actix_web::{web, App, HttpResponse, HttpServer};

pub mod middleware;
pub mod orderbook;
pub mod routes;
pub mod types;

use orderbook::Orderbook;
use types::user::{Balance, User};

pub struct AppState {
    pub next_user_id: Mutex<u32>,
    pub users: Mutex<Vec<User>>,
    pub usd_balances: Mutex<HashMap<u32, Balance>>,
    pub stock_balances: Mutex<HashMap<u32, HashMap<String, Balance>>>,
    pub orderbook: Mutex<Orderbook>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(AppState {
        next_user_id: Mutex::new(1),
        users: Mutex::new(Vec::new()),
        usd_balances: Mutex::new(HashMap::new()),
        stock_balances: Mutex::new(HashMap::new()),
        orderbook: Mutex::new(Orderbook::new("sol")),
    });
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(routes::user::signup)
            .service(routes::user::signin)
            .service(routes::user::balance)
            .service(routes::user::onramp)
            .service(routes::user::deposit)
            .service(routes::user::orderbook)
            .service(routes::user::orders)
            .service(routes::user::order)
            .service(routes::user::cancel)
            .route("/", web::get().to(|| async { HttpResponse::Ok().content_type("text/html; charset=utf-8").body(include_str!("public/index.html")) }))
    })
    .bind(("127.0.0.1", 3001))?
    .run()
    .await
}
