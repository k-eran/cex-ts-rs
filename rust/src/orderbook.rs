use std::collections::BTreeMap;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Order { pub order_id: u64, pub user_id: u32, pub side: String, pub price: f64, pub qty: f64, pub filled_qty: f64, pub created_at: u64 }
#[derive(Serialize)]
pub struct Fill { pub buyer: u32, pub seller: u32, pub price: f64, pub qty: f64 }
#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Event { Fill(Fill), Resting { user_id: u32, order_id: u64, side: String, price: f64, qty: f64 } }
#[derive(Serialize)]
pub struct Level { pub price: f64, pub qty: f64, pub orders: usize }
#[derive(Serialize)]
pub struct Snapshot { pub symbol: String, pub bids: Vec<Level>, pub asks: Vec<Level> }

pub struct Orderbook { pub symbol: String, next_id: u64, pub bids: BTreeMap<u64, Vec<Order>>, pub asks: BTreeMap<u64, Vec<Order>> }

fn key(price: f64) -> u64 { price.to_bits() }
fn price(key: u64) -> f64 { f64::from_bits(key) }

impl Orderbook {
    pub fn new(symbol: &str) -> Self { Self { symbol: symbol.to_string(), next_id: 1, bids: BTreeMap::new(), asks: BTreeMap::new() } }
    pub fn add(&mut self, user_id: u32, side: &str, limit: f64, qty: f64) -> Vec<Event> {
        let mut events = Vec::new(); let mut remaining = qty;
        if side == "bid" {
            let prices: Vec<u64> = self.asks.keys().copied().collect();
            for key in prices { let match_price = price(key); if match_price > limit { break; } if let Some(bucket) = self.asks.get_mut(&key) {
                for resting in bucket.iter_mut() { let available = resting.qty - resting.filled_qty; let traded = remaining.min(available); if traded <= 0.0 { continue; } resting.filled_qty += traded; remaining -= traded; events.push(Event::Fill(Fill { buyer: user_id, seller: resting.user_id, price: match_price, qty: traded })); if remaining <= 0.0 { break; } }
                bucket.retain(|order| order.filled_qty < order.qty);
            } if self.asks.get(&key).map_or(false, |b| b.is_empty()) { self.asks.remove(&key); } if remaining <= 0.0 { break; } }
        } else {
            let prices: Vec<u64> = self.bids.keys().rev().copied().collect();
            for key in prices { let match_price = price(key); if match_price < limit { break; } if let Some(bucket) = self.bids.get_mut(&key) {
                for resting in bucket.iter_mut() { let available = resting.qty - resting.filled_qty; let traded = remaining.min(available); if traded <= 0.0 { continue; } resting.filled_qty += traded; remaining -= traded; events.push(Event::Fill(Fill { buyer: resting.user_id, seller: user_id, price: match_price, qty: traded })); if remaining <= 0.0 { break; } }
                bucket.retain(|order| order.filled_qty < order.qty);
            } if self.bids.get(&key).map_or(false, |b| b.is_empty()) { self.bids.remove(&key); } if remaining <= 0.0 { break; } }
        }
        if remaining > 0.0 {
            let order = Order { order_id: self.next_id, user_id, side: side.to_string(), price: limit, qty, filled_qty: 0.0, created_at: self.next_id }; self.next_id += 1;
            let book = if side == "bid" { &mut self.bids } else { &mut self.asks }; book.entry(key(limit)).or_default().push(order.clone());
            events.push(Event::Resting { user_id, order_id: order.order_id, side: order.side, price: limit, qty: remaining });
        } events
    }
    pub fn cancel(&mut self, user_id: u32, order_id: u64) -> Option<Order> {
        for book in [&mut self.bids, &mut self.asks] {
            let found = book.iter().find_map(|(key, orders)| {
                orders.iter().position(|o| o.user_id == user_id && o.order_id == order_id).map(|pos| (*key, pos))
            });
            if let Some((key, pos)) = found {
                let orders = book.get_mut(&key).unwrap();
                let order = orders.remove(pos);
                if orders.is_empty() { book.remove(&key); }
                return Some(order);
            }
        }
        None
    }
    pub fn user_orders(&self, user_id: u32) -> Vec<Order> { self.bids.values().chain(self.asks.values()).flat_map(|orders| orders.iter().filter(|o| o.user_id == user_id).cloned()).collect() }
    pub fn snapshot(&self) -> Snapshot { fn levels(book: &BTreeMap<u64, Vec<Order>>, reverse: bool) -> Vec<Level> { let mut result: Vec<Level> = book.iter().map(|(key, orders)| Level { price: price(*key), qty: orders.iter().map(|o| o.qty - o.filled_qty).sum(), orders: orders.len() }).collect(); if reverse { result.reverse(); } result } Snapshot { symbol: self.symbol.clone(), bids: levels(&self.bids, true), asks: levels(&self.asks, false) } }
}
