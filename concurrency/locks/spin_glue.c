#include <stdio.h>

#define locked   1
#define unlocked 0

extern void lock_mutex(void * mutex);
extern void unlock_mutex(void * mutex);

unsigned int output_mutex = unlocked;

int putstr(char * str){
    int i;

    /* Wait until the output mutex is acquired */
    lock_mutex(&output_mutex);

    /* Entered critical section */
    printf("this is some useless output\n");

    /* Leave critical section - release output mutex */
    unlock_mutex(&output_mutex);

    return i;
}


int main(int argc, char *argv[]){
    char *s = "hey there";
    putstr(s);
    return 0;
}