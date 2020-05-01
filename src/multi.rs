use std::cell::UnsafeCell;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{spin_loop_hint, AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

struct WaitFuture(CountDownLatch);

unsafe impl Send for CountDownLatch {}
unsafe impl Sync for CountDownLatch {}

impl Future for WaitFuture {
    type Output = Result<(), ()>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = Pin::get_mut(self);
        this.0.with_lock(|latch| {
            if latch.count.load(Ordering::SeqCst) > 0 {
                let waiters = unsafe { &mut *latch.waiters.get() };
                waiters.push(cx.waker().clone());
                Poll::Pending
            } else {
                Poll::Ready(Ok(()))
            }
        })
    }
}
impl CountDownLatch {
    pub fn new(count: usize) -> CountDownLatch {
        let counter = Arc::new(AtomicUsize::new(count));
        let locked = Arc::new(AtomicBool::new(false));

        CountDownLatch {
            locked: locked,
            count: counter,
            waiters: Arc::new(UnsafeCell::new(Vec::with_capacity(count))),
        }
    }

    pub async fn count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }

    pub fn wait(&self) -> impl Future<Output = Result<(), ()>> {
        WaitFuture(self.clone())
    }

    pub fn with_lock<T, R>(&self, cb: T) -> R
    where
        T: FnOnce(&CountDownLatch) -> R,
    {
        while self.locked.compare_and_swap(false, true, Ordering::Acquire) != false {
            while self.locked.load(Ordering::Relaxed) {
                spin_loop_hint();
            }
        }

        let ret = cb(self);

        self.locked.store(false, Ordering::Release);
        ret
    }

    pub async fn count_down(&self) -> Result<(), ()> {
        self.with_lock(|latch| {
            if latch.count.load(Ordering::SeqCst) > 0 {
                let prev = latch.count.fetch_sub(1, Ordering::SeqCst);
                if prev == 1 {
                    let waiters = unsafe { &mut *latch.waiters.get() };
                    while let Some(e) = waiters.pop() {
                        e.wake();
                    }
                }
            }
        });
        Ok(())
    }
}

#[derive(Clone)]
pub struct CountDownLatch {
    locked: Arc<AtomicBool>,
    count: Arc<AtomicUsize>,
    waiters: Arc<UnsafeCell<Vec<Waker>>>,
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
