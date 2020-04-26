use async_std::{self, task};
use countdownlatch::multi::CountDownLatch;
use std::time::Duration;


#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let latch = CountDownLatch::new(100);

    for _ in 0..100 {
        let latch1 = latch.clone();
        task::spawn(async move {
            latch1.wait().await.unwrap();
        });
    }


    for _ in 0..100 {
        let latch1 = latch.clone();
        task::spawn(async move {
            latch1.count_down().await.unwrap();
        });
    }

    
   
    println!("Final wait");
    latch.wait().await.unwrap();
    
    task::sleep(Duration::from_secs(1)).await;
    

    Ok(())
}
