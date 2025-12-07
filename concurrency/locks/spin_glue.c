#include <stdio.h>
#include <pthread.h>
#include <assert.h>

#define locked   1
#define unlocked 0

extern void lock_mutex(void * mutex);
extern void unlock_mutex(void * mutex);

unsigned int output_mutex = unlocked;
static int val = 0;
const int MAX_NUM = 100;
const int MAX_THREAD_LIMIT = 15;

void *worker(void *args) {
    /* Wait until the output mutex is acquired */
    lock_mutex(&output_mutex);

    for (int i = 0; i < MAX_NUM; i++){
        val++;
    }

    /* Leave critical section - release output mutex */
    unlock_mutex(&output_mutex);
    return NULL;
}


int main(int argc, char *argv[]){
    pthread_t t[MAX_THREAD_LIMIT];

    for(int i = 0; i < MAX_THREAD_LIMIT; i++){
        assert(pthread_create(&t[i], NULL, worker, NULL) ==0);
    }


    for (int i = 0; i < MAX_THREAD_LIMIT; i++){
        assert(pthread_join(t[i], NULL)==0);
    }

    printf("val = %d\n", val);
    return 0;
}