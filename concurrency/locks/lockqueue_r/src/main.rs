mod simple_queue;
mod parking_lot;

use std::sync::Arc;
use std::thread;
use parking_lot::Mutex;



fn mutex_eg(){
    // For some reason this deadlocks in some cases, Would be
    // nice to add some deadlock detection
    let m = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    for i in 0..4000{
        let m = m.clone();
        let handle = thread::spawn(move || {
            let mut guard = m.lock();
            *guard += 1;
        });

        handles.push(handle)

    }

    for handle in handles{
        handle.join().unwrap();
    }

    let guard = m.lock();
    println!("Final value: {}", *guard)

}




fn main() {
    // let spinlock = Arc::new(Mutex::new());
    // let counter = Arc::new(AtomicU32::new(0));
    // // let mut counter = 0;
    // let mut handles = vec![];
    // for i in 0..100 {
    //     let mut counter = counter.clone();
    //     let lock = spinlock.clone();
    //     let handle = thread::spawn(move || {
    //         lock.lock();
    //         println!("Thread {} acquired the lock", i);
    //         let old = counter.load(Ordering::SeqCst);
    //         counter.store(old + 1, Ordering::SeqCst);
    //
    //         thread::sleep(std::time::Duration::from_millis(100));
    //         println!("Thread {} releasing the lock", i);
    //         lock.unlock();
    //     });
    //     handles.push(handle);
    // }
    // for handle in handles {
    //     handle.join().unwrap();
    // }
    //
    // let final_value = counter.load(Relaxed);
    // println!("Final value: {}", final_value);
    mutex_eg();
}
