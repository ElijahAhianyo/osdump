    .equ locked, 1
    .equ unlocked, 0
    .section .text
    .type lock_mutex, %function
    .global lock_mutex
lock_mutex:
// x0   address of lock
1:
    ldaxr w1, [x0]          // exclusive acquire
    cbnz w1, 2f             // check if lock is 0(we want to it to be 0 to acquire it)
    mov w2, #1
    stlxr  w3, w2, [x0]     // exclusive store-release
    cbnz w3, 1b             // retry if we failed to get the lock
    // we have acquired the lock!
    ret

2:
    wfe                     // park until some thread wakes us up
    b 1b                    // someone woke us up, so we try to acquire lock again


    .section .text
    .type unlock_mutex, %function
    .global unlock_mutex
unlock_mutex:
    mov w1, #0
    stlr w1, [x0]

    dsb ish                 // ensure all stores are complete before another thread wakes up. This is required to 
                            // prevent stale data
    sev                     // wake up any parked thread
    ret
