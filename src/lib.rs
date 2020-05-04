//! This library provide an implementation of an async [`CountDownLatch`],
//! which keeps a counter syncronized via [`Mutex`] in it's internal state and allows tasks to wait until
//! the counter reaches zero.
//!
//! # Example
//! ```rust,no_run
//! use wait_for_me::CountDownLatch;
//! use smol::{self,Task};
//! fn main() -> Result<(), Box<std::error::Error>> {
//!    smol::run(async {
//!         let latch = CountDownLatch::new(1);
//!         let latch1 = latch.clone();
//!         Task::spawn(async move {
//!             latch1.count_down().await;
//!         }).detach();
//!         latch.wait().await;
//!         Ok(())
//!    })
//!
//!}
//! ```
//! [`Mutex`][piper::Mutex]

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
    /// Creates a new [`CountDownLatch`] with a given count.
    pub fn new(count: usize) -> CountDownLatch {
        CountDownLatch {
            state: Arc::new(Mutex::new(CountDownState {
                count,
                waiters: vec![],
            })),
        }
    }

    /// Returns the current count.
    pub async fn count(&self) -> usize {
        let state = self.state.lock();
        state.count
    }

    /// Cause the current task to wait until the counter reaches zero
    pub fn wait(&self) -> impl Future<Output = ()> {
        WaitFuture(self.clone())
    }

    /// Decrement the counter of one unit. If the counter reaches zero all the waiting tasks are released.
    pub async fn count_down(&self) {
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
    }
}

struct WaitFuture(CountDownLatch);

impl Future for WaitFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.0.state.lock();
        if state.count > 0 {
            state.waiters.push(cx.waker().clone());
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
/// A syncronization primitive that allow one or more tasks to wait untile the given counter reaches zero.
/// This is an async port of [CountDownLatch](https://docs.oracle.com/javase/7/docs/api/java/util/concurrent/CountDownLatch.html) in Java.
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
            .spawn(async move { latch1.count_down().await })
            .unwrap();

        let latch2 = latch.clone();
        spawner
            .spawn(async move { latch2.count_down().await })
            .unwrap();

        let latch3 = latch.clone();
        spawner
            .spawn(async move {
                latch3.wait().await;
            })
            .unwrap();

        spawner
            .spawn(async move {
                latch.wait().await;
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
                .spawn(async move { latch1.count_down().await })
                .unwrap();
        }

        for _ in 0..100 {
            let latch1 = latch.clone();
            spawner.spawn(async move { latch1.wait().await }).unwrap();
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
                .spawn(async move { latch1.count_down().await })
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
                .spawn(async move { latch1.count_down().await })
                .unwrap();
        }

        pool.run();

        for _ in 0..100 {
            let latch1 = latch.clone();
            spawner.spawn(async move { latch1.wait().await }).unwrap();
        }

        pool.run();
    }

    #[test]
    fn latch_count_test() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let mut pool = LocalPool::new();
        let pre_counter = Arc::new(AtomicUsize::new(0));
        let post_counter = Arc::new(AtomicUsize::new(0));

        let spawner = pool.spawner();
        let latch = CountDownLatch::new(1);

        let latch1 = latch.clone();
        let pre_counter1 = pre_counter.clone();
        let post_counter1 = post_counter.clone();
        spawner
            .spawn(async move {
                pre_counter1.store(latch1.count().await, Ordering::Relaxed);
                latch1.count_down().await;
                post_counter1.store(latch1.count().await, Ordering::Relaxed);
            })
            .unwrap();

        pool.run();

        assert_eq!(1, pre_counter.load(Ordering::Relaxed));
        assert_eq!(0, post_counter.load(Ordering::Relaxed));
    }
}
