/*
This shows a simple implementation of a spin lock using load-acquire/store-release.
Unlike the conversional spin locks that busy-loops until the condition is true(or the lock
is acquired), make use of the WFE(Wait for Event) instruction from the hardware which parks the
CPU. This bypasses the OS scheduler and it saves power doing so. To the OS, the thread is still 
running, however to the hardware, the CPU is idle. There's no context switch. 
Once the lock is available, we send a signal via sev(signal event) which wakes up the waiting CPUs.
A spurious wake(or thuderhead) shouldnt be a problem since this is dependednt on the number of CPUs.

While this seems like less work(no context switching, etc), we do hog the CPU, so the OS will not be 
able to schedule a thread on a core with a WFE thread.

The glue code to run this in located in `spin_glue.c`. 

Compile using gcc to run:

```
gcc -Wall -g spin_glue.c spin.s -o spin
```

run:

```
./spin
```
*/

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
