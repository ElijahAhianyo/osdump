use core::sync::atomic::AtomicPtr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::{hint, thread};


struct Spinlock {
    lock: AtomicBool
}

impl Spinlock {
    fn new() -> Self{
        Self{
            lock: AtomicBool::new(false)
        }
    }

    fn lock(&self){
        for _ in 0..10{
            if self.lock.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed ).is_ok(){
                return
            }
        }
        thread::yield_now()
    }



    fn unlock(&self) {
        self.lock.store(false, Ordering::Release);
    }
}




fn main() {
    let spinlock = Arc::new(Spinlock::new());
    let counter = Arc::new(AtomicU32::new(0));
    let mut handles = vec![];
    for i in 0..100 {
        let counter = counter.clone();
        let lock = spinlock.clone();
        let handle = thread::spawn(move || {
            lock.lock();
            println!("Thread {} acquired the lock", i);
            let old = counter.load(Ordering::SeqCst);
            counter.store(old + 1, Ordering::SeqCst);

            thread::sleep(std::time::Duration::from_millis(100));
            println!("Thread {} releasing the lock", i);
            lock.unlock();
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }

    let final_value = counter.load(Ordering::Relaxed);
    println!("Final value: {}", final_value);
}
