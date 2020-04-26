use async_std::{self, task};
use countdownlatch::oneshot::latch;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (waiter, latch) = latch(100);

    for _ in 0..100 {
        let latch1 = latch.clone();
        task::spawn(async move {
            latch1.count_down().await.unwrap();
        });
    }
    waiter.wait().await.unwrap();

    Ok(())
}
