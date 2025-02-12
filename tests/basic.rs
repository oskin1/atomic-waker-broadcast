use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::task::Poll;
use std::thread;

use atomic_waker::AtomicWaker;
use futures::executor::block_on;
use futures::future::poll_fn;

#[test]
fn basic() {
    let atomic_waker = Arc::new(AtomicWaker::new());
    let atomic_waker_copy = atomic_waker.clone();

    let returned_pending = Arc::new(AtomicUsize::new(0));
    let returned_pending_copy = returned_pending.clone();

    let woken = Arc::new(AtomicUsize::new(0));
    let woken_copy = woken.clone();

    let t = thread::spawn(move || {
        let mut pending_count = 0;

        block_on(poll_fn(move |cx| {
            if woken_copy.load(Ordering::Relaxed) == 1 {
                Poll::Ready(())
            } else {
                // Assert we return pending exactly once
                assert_eq!(0, pending_count);
                pending_count += 1;
                atomic_waker_copy.try_register(cx.waker());

                returned_pending_copy.store(1, Ordering::Relaxed);

                Poll::Pending
            }
        }))
    });

    while returned_pending.load(Ordering::Relaxed) == 0 {}

    // give spawned thread some time to sleep in `block_on`
    thread::yield_now();

    woken.store(1, Ordering::Relaxed);
    atomic_waker.wake();

    t.join().unwrap();
}

#[test]
fn multiple() {
    let atomic_waker = Arc::new(AtomicWaker::new());
    let atomic_waker_copy_1 = atomic_waker.clone();
    let atomic_waker_copy_2 = atomic_waker.clone();

    let returned_pending = Arc::new(AtomicUsize::new(0));
    let returned_pending_copy_1 = returned_pending.clone();
    let returned_pending_copy_2 = returned_pending.clone();

    let woken = Arc::new(AtomicUsize::new(0));
    let woken_copy_1 = woken.clone();
    let woken_copy_2 = woken.clone();

    let t1 = thread::spawn(move || {
        let mut pending_count = 0;

        block_on(poll_fn(move |cx| {
            if woken_copy_1.load(Ordering::Relaxed) == 1 {
                Poll::Ready(())
            } else {
                // Assert we return pending exactly once
                assert_eq!(0, pending_count);
                pending_count += 1;
                atomic_waker_copy_1.try_register(cx.waker());

                returned_pending_copy_1.store(1, Ordering::Relaxed);

                Poll::Pending
            }
        }))
    });

    let t2 = thread::spawn(move || {
        let mut pending_count = 0;

        block_on(poll_fn(move |cx| {
            if woken_copy_2.load(Ordering::Relaxed) == 1 {
                Poll::Ready(())
            } else {
                // Assert we return pending exactly once
                assert_eq!(0, pending_count);
                pending_count += 1;
                atomic_waker_copy_2.try_register(cx.waker());

                returned_pending_copy_2.store(2, Ordering::Relaxed);

                Poll::Pending
            }
        }))
    });

    while returned_pending.load(Ordering::Relaxed) <= 1 {}

    // give spawned thread some time to sleep in `block_on`
    thread::yield_now();

    woken.store(1, Ordering::Relaxed);
    atomic_waker.wake();

    t1.join().unwrap();
    t2.join().unwrap();
}
