extern crate core;

use core::fmt;
use std::cell::UnsafeCell;
use std::collections::{HashMap, VecDeque};
use std::fmt::Formatter;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, OnceLock};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::thread::{self, Thread};
use std::sync::Mutex as StdMutex;
use std::time::Duration;

const LOCKED_BIT: u8 = 0b01;
const PARKED_BIT: u8 = 0b10;


struct Waiter{
    thread: Thread,
    notified: AtomicBool
}

type WaiterQueue = VecDeque<Arc<Waiter>>;



struct ParkingLot {
    map: StdMutex<HashMap<usize, WaiterQueue>>,
}


impl ParkingLot {
    fn new() -> Self {
        Self {
            map: StdMutex::new(HashMap::new())
        }
    }

    // This assumes the caller has the PARKED_BIT set on the mutex before calling
    fn park_current_thread(&self, key: usize) {
        let waiter = Arc::new( Waiter{
            thread: thread::current(),
            notified: AtomicBool::new(false)
        });

        // We insert the waiter into the queue under the global lock, then we release before parking.
        // This should avoid missed wakeups  because `unlock` will take the same lock before dequeuing.
        let mut map = self.map.lock().unwrap();
        let q = map.entry(key).or_insert_with(VecDeque::new);
        q.push_back(waiter.clone());
        drop(map);

        // In the case where a thread is unparked before we actually park, we set the notified flag so the
        // unpark is not missed.
        while !waiter.notified.load(Ordering::Acquire){
            thread::park_timeout(Duration::from_millis(100))
        }

    }

    fn unpark_one(&self, key: usize, callback: impl FnOnce(UnparkResult) -> ()) -> UnparkResult {
        let mut result = UnparkResult::default();
        let waiter = {
            let mut map = self.map.lock().unwrap();
            if let Some(queue) = map.get_mut(&key) {
                let w = queue.pop_front();
                if queue.is_empty(){
                    // remove queue to keep the map small
                    map.remove(&key);
                    result.have_more_threads = false;
                }
                else {
                    result.have_more_threads = true;
                }
                w
            } else{
                None
            }

        };

        if let Some(waiter) = waiter {
            let map = self.map.lock();
            // notify and unpark the thread
            waiter.notified.store(true, Ordering::Release);
            callback(result);

            waiter.thread.unpark();
            drop(map);
            result.unparked_threads = 1;

        } else{
            // callback must be run under the lock
            let map = self.map.lock().unwrap();
            callback(result);

            drop(map);
            result.unparked_threads = 0;
        }
        result


    }
}

fn parking_lot() -> &'static ParkingLot {
    static PL: OnceLock<ParkingLot> = OnceLock::new();
    PL.get_or_init(|| ParkingLot::new())
}


struct RawMutex{
    /// This atomic integer holds the current state of the mutex instance. Only the two lowest bits
    /// are used. See `LOCKED_BIT` and `PARKED_BIT` for the bitmask for these bits.
    ///
    /// # State table:
    ///
    /// PARKED_BIT | LOCKED_BIT | Description
    ///     0      |     0      | The mutex is not locked, nor is anyone waiting for it.
    /// -----------+------------+------------------------------------------------------------------
    ///     0      |     1      | The mutex is locked by exactly one thread. No other thread is
    ///            |            | waiting for it.
    /// -----------+------------+------------------------------------------------------------------
    ///     1      |     0      | The mutex is not locked. One or more thread is parked or about to
    ///            |            | park. At least one of the parked threads are just about to be
    ///            |            | unparked, or a thread heading for parking might abort the park.
    /// -----------+------------+------------------------------------------------------------------
    ///     1      |     1      | The mutex is locked by exactly one thread. One or more thread is
    ///            |            | parked or about to park, waiting for the lock to become available.
    state: AtomicU8
}

unsafe impl Send for RawMutex{}
unsafe  impl Sync for RawMutex{}

impl RawMutex{
    const fn new() -> Self {
        Self{
            state: AtomicU8::new(0)
        }
    }

    fn try_lock(&self) -> bool {
        self.state.compare_exchange(0, LOCKED_BIT, Ordering::Acquire, Ordering::Relaxed).is_ok()
    }

    fn lock(&self) {
        if self.state.compare_exchange(0, LOCKED_BIT, Ordering::Acquire, Ordering::Relaxed).is_ok(){
            // println!("locked fast!");
            return
        }
        // println!("locksed slow!");
        self.lock_slow();
        // println!("finished locked fast");
    }

    fn lock_slow(&self) {

        // spin for a short while to see if we can acquire the lock
        const SPIN_ITERS: usize = 40;
        for _ in 0..SPIN_ITERS {
            std::hint::spin_loop();
            if self.state.compare_exchange(0, LOCKED_BIT, Ordering::Acquire, Ordering::Relaxed).is_ok(){
                return
            }
        }

        // we couldnt get the lock, time to sleep

        // use the RawMutex object address as the key
        let key = self as *const _ as usize;

        // indicate that there are (or will be ) parked waiters
        self.state.fetch_or(PARKED_BIT, Ordering::AcqRel);

        parking_lot().park_current_thread(key);

        // After waking up, try to acquire the lock until we succeed

        loop {
            if self.state.compare_exchange(0, LOCKED_BIT, Ordering::Acquire, Ordering::Relaxed).is_ok(){
                return
            }

            std::hint::spin_loop();
        }
    }


    fn unlock(&self) {
        // if self.state.compare_exchange(LOCKED_BIT, 0, Ordering::Acquire, Ordering::Relaxed).is_ok(){
        //     // println!("unlocked!");
        //     return;
        // }
        let prev = self.state.fetch_and(0, Ordering::Release);
        if prev & PARKED_BIT == 0 {
            return
        }
        // println!("start unlock slow!");
        self.unlock_slow();
        // println!("unlocked slow done!")
    }

    fn unlock_slow(&self) {
        let addr = self as *const _ as usize;

        let callback = |result: UnparkResult| {
                // If there are no threads, we clear locked bit
                if result.have_more_threads {
                    self.state.store(PARKED_BIT, Ordering::Release)
                }
            else {
                self.state.store(0, Ordering::Release);
            }
        };

        parking_lot().unpark_one(addr, callback);
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct UnparkResult{
    pub unparked_threads: usize,
    pub have_more_threads: bool
}


pub struct Mutex<T: ?Sized> {
    raw: RawMutex,
    value: UnsafeCell<T>
}

unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Send for Mutex<T>{}


impl<T> Mutex<T> {
    pub(crate) const fn new(value: T) -> Self{
        Self{
            value: UnsafeCell::new(value),
            raw: RawMutex::new()
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.raw.lock();
        MutexGuard { mutex: self}

    }

    fn try_lock(&self) -> Option<MutexGuard<'_, T>>{
        if self.raw.try_lock(){
            Some(MutexGuard{mutex: self})
        } else {
            None
        }
    }

    pub fn raw_ptr(&mut self) -> *mut T{
        self.value.get()
    }
}




pub struct MutexGuard<'a, T: ?Sized> {
    mutex: &'a Mutex<T>
}

unsafe impl<'a, T: ?Sized + Send> Send for MutexGuard<'a, T> {}

impl <'a, T: ?Sized> Deref for MutexGuard<'a, T>{
    type Target = T;

    fn deref(&self) -> &Self::Target {
       unsafe { &*self.mutex.value.get() }
    }
}

impl <'a, T: ?Sized> DerefMut for MutexGuard<'a, T>{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {&mut *self.mutex.value.get()}
    }
}

impl <'a, T: ?Sized> Drop for MutexGuard<'a, T>{
    fn drop(&mut self) {
        self.mutex.raw.unlock()
    }
}


impl<T: fmt::Debug> fmt::Debug for Mutex<T>{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mutex").finish_non_exhaustive()
    }
}

fn main() {


}