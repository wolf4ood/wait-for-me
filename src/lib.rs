#![forbid(unsafe_code)]
use std::future::Future;
use std::sync::{Arc, Mutex};

use std::pin::Pin;
use std::task::{Context, Poll, Waker};

pub fn latch(count: usize) -> (Waiter, Latch) {
    let count = Arc::new(Mutex::new(count));
    let latch = Latch {
        count: count.clone(),
        waker: Arc::new(Mutex::new(None)),
    };
    (
        Waiter {
            count: count.clone(),
            latch: latch.clone(),
        },
        latch,
    )
}

pub struct Waiter {
    count: Arc<Mutex<usize>>,
    latch: Latch,
}

impl Waiter {
    pub fn wait(self) -> impl Future<Output = Result<(), ()>> {
        WaiterFuture(self)
    }
}

struct WaiterFuture(Waiter);

impl Future for WaiterFuture {
    type Output = Result<(), ()>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let val = self.0.count.lock().unwrap();
        if *val == 0 {
            Poll::Ready(Ok(()))
        } else {
            let mut guard = self.0.latch.waker.lock().unwrap();
            guard.replace(cx.waker().clone());
            Poll::Pending
        }
    }
}

#[derive(Clone)]
pub struct Latch {
    count: Arc<Mutex<usize>>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl Latch {
    pub fn count_down(self) {
        let mut val = self.count.lock().unwrap();
        match *val {
            1 => {
                *val -= 1;
                drop(val);
                let mut guard = self.waker.lock().unwrap();
                match guard.take() {
                    Some(waker) => waker.wake(),
                    None => {}
                }
            }
            n @ _ if n > 0 => {
                *val -= 1;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {

    use super::latch;
    use futures_executor::LocalPool;
    use futures_util::task::SpawnExt;

    #[test]
    fn latch_test() {
        let mut pool = LocalPool::new();

        let spawner = pool.spawner();

        let (waiter, latch) = latch(2);

        let latch1 = latch.clone();
        spawner
            .spawn(async move { latch1.clone().count_down() })
            .unwrap();

        spawner.spawn(async move { latch.count_down() }).unwrap();

        spawner
            .spawn(async move {
                waiter.wait().await.unwrap();
            })
            .unwrap();

        pool.run();
    }
}
