use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
enum Order {
    Cancel { order_id: u64 },           // Highest priority
    Market { symbol: String, qty: f64 }, // High priority
    Limit { symbol: String, price: f64, qty: f64 }, // Normal priority
}

#[tokio::main]
async fn main() {
    let (cancel_tx, mut cancel_rx) = mpsc::channel::<Order>(100);
    let (market_tx, mut market_rx) = mpsc::channel::<Order>(100);
    let (limit_tx, mut limit_rx) = mpsc::channel::<Order>(100);
    
    // Simulate incoming orders
    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        limit_tx.send(Order::Limit { 
            symbol: "BTC/USD".into(), 
            price: 45000.0, 
            qty: 0.1 
        }).await.ok();
        
        sleep(Duration::from_millis(10)).await;
        market_tx.send(Order::Market { 
            symbol: "ETH/USD".into(), 
            qty: 1.0 
        }).await.ok();
        
        sleep(Duration::from_millis(5)).await;
        cancel_tx.send(Order::Cancel { 
            order_id: 12345 
        }).await.ok();
    });
    
    // Order matching engine
    let mut processed = 0;
    loop {
        tokio::select! {
            biased;  // ← Critical: Cancel orders must be processed first!
            
            // Process cancellations immediately
            Some(order) = cancel_rx.recv() => {
                println!("🚨 CANCEL: {:?}", order);
                processed += 1;
            }
            
            // Then market orders (immediate execution)
            Some(order) = market_rx.recv() => {
                println!("⚡ MARKET: {:?}", order);
                processed += 1;
            }
            
            // Finally limit orders (can wait)
            Some(order) = limit_rx.recv() => {
                println!("📋 LIMIT: {:?}", order);
                processed += 1;
            }
            
            // Timeout after 1 second
            _ = sleep(Duration::from_secs(1)) => {
                break;
            }
        }
        
        if processed >= 3 {
            sleep(Duration::from_millis(100)).await;
        }
    }
}

// Output:
// 📋 LIMIT: Limit { symbol: "BTC/USD", price: 45000.0, qty: 0.1 }
// ⚡ MARKET: Market { symbol: "ETH/USD", qty: 1.0 }
// 🚨 CANCEL: Cancel { order_id: 12345 }