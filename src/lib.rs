use piper::Mutex;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

struct CountDownState {
    count: usize,
    waiters: Vec<Waker>,
}

impl CountDownLatch {
    pub fn new(count: usize) -> CountDownLatch {
        CountDownLatch {
            state: Arc::new(Mutex::new(CountDownState {
                count,
                waiters: vec![],
            })),
        }
    }

    pub async fn count(&self) -> usize {
        let state = self.state.lock();
        state.count
    }

    pub fn wait(&self) -> impl Future<Output = Result<(), ()>> {
        WaitFuture(self.clone())
    }

    pub async fn count_down(&self) -> Result<(), ()> {
        let mut state = self.state.lock();

        match state.count {
            1 => {
                state.count -= 1;
                while let Some(e) = state.waiters.pop() {
                    e.wake();
                }
            }
            n @ _ if n > 0 => {
                state.count -= 1;
            }
            _ => {}
        };

        Ok(())
    }
}

struct WaitFuture(CountDownLatch);

impl Future for WaitFuture {
    type Output = Result<(), ()>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.0.state.lock();
        if state.count > 0 {
            state.waiters.push(cx.waker().clone());
            Poll::Pending
        } else {
            Poll::Ready(Ok(()))
        }
    }
}

#[derive(Clone)]
pub struct CountDownLatch {
    state: Arc<Mutex<CountDownState>>,
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

    #[test]
    fn multi_latch_no_wait_test() {
        let mut pool = LocalPool::new();

        let spawner = pool.spawner();
        let latch = CountDownLatch::new(100);

        for _ in 0..200 {
            let latch1 = latch.clone();
            spawner
                .spawn(async move { latch1.count_down().await.unwrap() })
                .unwrap();
        }

        pool.run();
    }

    #[test]
    fn multi_latch_post_wait_test() {
        let mut pool = LocalPool::new();

        let spawner = pool.spawner();
        let latch = CountDownLatch::new(100);

        for _ in 0..200 {
            let latch1 = latch.clone();
            spawner
                .spawn(async move { latch1.count_down().await.unwrap() })
                .unwrap();
        }

        pool.run();

        for _ in 0..100 {
            let latch1 = latch.clone();
            spawner
                .spawn(async move { latch1.wait().await.unwrap() })
                .unwrap();
        }

        pool.run();
    }
}
