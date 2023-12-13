use std::time::Duration;
use tokio::time::sleep;
use tokio::{self, task};
use wait_for_me::CountDownLatch;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let latch = CountDownLatch::new(10);
    for _ in 0..10 {
        let latch1 = latch.clone();
        task::spawn(async move {
            sleep(Duration::from_secs(3)).await;
            latch1.count_down().await;
        });
    }

    let result = latch.wait_for(Duration::from_secs(1)).await;
    assert_eq!(false, result);

    Ok(())
}
