use async_std::{self, task};
use countdownlatch::CountDownLatch;



#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let latch = CountDownLatch::new(10);
    for _ in 0..10 {
        let latch1 = latch.clone();
        task::spawn(async move {
            latch1.count_down().await.unwrap();
        });
    }
    latch.wait().await.unwrap();


    Ok(())
}

