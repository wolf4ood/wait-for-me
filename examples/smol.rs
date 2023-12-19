use wait_for_me::CountDownLatch;

#[smol_potat::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let latch = CountDownLatch::new(10);
    for _ in 0..10 {
        let latch1 = latch.clone();
        smol::spawn(async move {
            latch1.count_down().await;
        })
        .detach();
    }
    latch.wait().await;

    Ok(())
}
