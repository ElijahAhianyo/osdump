/*
This is meant to show a simple implementation of a compare-and-swap lock-free interface.

We exclusively retrieve or load a value from a location and compare that with the expected
If the value we loaded is different from the expected, we simply return the old value. It is up to 
the caller to handle the output of that. Otherwise, we proceed to store the new value at the location.
Note that ARM is weakly ordered, which means they can spuriously fail when trying to store exclusively for
reasons other than the value changing(such as interrupts, etc), so we loop till this either works or the value
changes.

*/

    .section .text
    .type compare_and_swap, %function
    .global compare_and_swap
compare_and_swap:
    /* Args:
        x0      pointer
        x1      expected value
        x2      new value
    */
1:
    ldxr x3, [x0]
    cmp x1, x3
    bne 2f
    stlxr w4, x2, [x0]  // w4 = 1 means failed, 0 = success
    cbnz w4, 1b             // Try again if this failed
2:
    mov x0, x3           // return the old value at the provided location
    ret 
