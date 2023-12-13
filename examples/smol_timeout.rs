use smol::Timer;
use std::time::Duration;
use wait_for_me::CountDownLatch;

#[smol_potat::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let latch = CountDownLatch::new(10);
    for _ in 0..10 {
        let latch1 = latch.clone();
        smol::spawn(async move {
            Timer::after(Duration::from_secs(3)).await;
            latch1.count_down().await;
        })
        .detach();
    }
    let result = latch.wait_for(Duration::from_secs(1)).await;

    assert_eq!(false, result);

    Ok(())
}
