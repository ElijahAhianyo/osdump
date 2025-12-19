use std::collections::VecDeque;
use std::fs::read;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release, SeqCst};
use std::thread::{self, Thread};

#[derive(Clone, Debug)]
struct ThreadData{
    thread: Thread,
    ready: Arc<AtomicBool>
}

impl ThreadData{
    fn new(thread: Thread, ready: bool) -> Self {
        Self {
            thread,
            ready: Arc::new(AtomicBool::new(ready))
        }
    }
}

struct QueueLock{
    locked: AtomicBool,
    queue: Arc<Mutex<VecDeque<ThreadData>>>
}

impl QueueLock {
    fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
            queue: Arc::new(Mutex::new(VecDeque::new()))
        }
    }

    fn lock(&self) {
        if self.locked.compare_exchange(false, true, Acquire, Relaxed).is_ok() {
            return
        }
        self.lock_slow()
    }

    fn lock_slow(&self) {

        let mut guard = self.queue.lock().unwrap();
        let curr = thread::current();
        let mut t = ThreadData::new(curr.clone(), false);
        guard.push_back(t.clone());
        drop(guard);
        thread::park();

        t.ready.store(true, SeqCst);
        thread::park();
    }


    fn unlock(&self) {
        // let mut guard = self.queue.lock().unwrap();
        // loop{
        //     if let Some(t) = guard.pop_front(){
        //         // just pass the lock to the next thread
        //         let read = t.ready.load(SeqCst);
        //         if read{
        //             t.thread.unpark();
        //             break;
        //         } else {
        //             // add back to queue
        //             guard.push_back(t)
        //         }
        //
        //     } else {
        //         // queue is empty. It is now safe to release the lock
        //         self.locked.store(false, Release);
        //         break;
        //     }
        // }

        loop {
            // Pop one candidate (limit scope of the guard)
            let maybe_waiter = {
                let mut q = self.queue.lock().unwrap();
                q.pop_front()
            };

            match maybe_waiter {
                None => {
                    // No waiters -> release the lock and return
                    self.locked.store(false, Release);
                    return;
                }
                Some(waiter) => {
                    // If the waiter is ready -> hand off
                    if waiter.ready.load(SeqCst) {
                        // transfer ownership to waiter; do not clear locked
                        waiter.thread.unpark();
                        return;
                    } else {
                        // Not ready: requeue it at the back and try the next one.
                        {
                            let mut q = self.queue.lock().unwrap();
                            q.push_back(waiter);
                        }
                        // Give the waiting threads a chance to set ready before we loop
                        thread::yield_now();
                        // continue looping
                    }
                }
            }
        }

    }
}
