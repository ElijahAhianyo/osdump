/*
This is meant to show a simple impplementation of a sompare-and-swap lock free interface.
The idea should be similar to that of the spin lock, only that we dont loop. 
We exclusively retrieve or load a value from a location and and compare that with the expected
If they match, we (exclusively) set store the new value at the given location and return 1 on failure 
and 0 on success.

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
    mov x0, #0          // return 0 meaning success!
    mov w0, w4
    b done
2:
    mov x0, #1           // return 1 to the caller indicating we failed

done:
    ret 
