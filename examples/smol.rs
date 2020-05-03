use countdownlatch::CountDownLatch;
use smol::Task;


#[smol_potat::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let latch = CountDownLatch::new(10);
    for _ in 0..10 {
        let latch1 = latch.clone();
        Task::spawn(async move {
            latch1.count_down().await.unwrap();
        }).detach();
    }
    latch.wait().await.unwrap();


    Ok(())
}
