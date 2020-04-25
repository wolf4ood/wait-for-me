use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use broadcaster::BroadcastChannel;

impl CountDownLatch {
    pub fn new(count: usize) -> CountDownLatch {
        let channel = BroadcastChannel::new();

        let count = Arc::new(AtomicUsize::new(count));
        CountDownLatch {
            count: count,
            channel: channel,
        }
    }

    pub async fn count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }
    pub async fn wait(&self) -> Result<(), ()> {
        if self.count().await > 0 {
            self.channel.clone().recv().await;
        }
        Ok(())
    }

    pub async fn count_down(&self) -> Result<(), ()> {
        match self.count().await {
            n @ _ if n > 0 => {
                let next = n - 1;
                let prev = self.count.compare_and_swap(n, next, Ordering::SeqCst);
                if prev == 1 {
                    self.channel.send(&()).await.unwrap();
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct CountDownLatch {
    count: Arc<AtomicUsize>,
    channel: BroadcastChannel<()>,
}

#[cfg(test)]
mod tests {

    use super::CountDownLatch;
    use futures_executor::LocalPool;
    use futures_util::task::SpawnExt;

    #[test]
    fn multi_latch_test() {
        let mut pool = LocalPool::new();

        let spawner = pool.spawner();
        let latch = CountDownLatch::new(2);
        let latch1 = latch.clone();
        spawner
            .spawn(async move { latch1.count_down().await.unwrap() })
            .unwrap();

        let latch2 = latch.clone();
        spawner
            .spawn(async move { latch2.count_down().await.unwrap() })
            .unwrap();

        let latch3 = latch.clone();
        spawner
            .spawn(async move {
                latch3.wait().await.unwrap();
            })
            .unwrap();

        spawner
            .spawn(async move {
                latch.wait().await.unwrap();
            })
            .unwrap();

        pool.run();
    }

    #[test]
    fn multi_latch_concurrent_test() {
        let mut pool = LocalPool::new();

        let spawner = pool.spawner();
        let latch = CountDownLatch::new(100);

        for _ in 0..200 {
            let latch1 = latch.clone();
            spawner
                .spawn(async move { latch1.count_down().await.unwrap() })
                .unwrap();
        }

        for _ in 0..100 {
            let latch1 = latch.clone();
            spawner
                .spawn(async move { latch1.wait().await.unwrap() })
                .unwrap();
        }

        pool.run();
    }
}
