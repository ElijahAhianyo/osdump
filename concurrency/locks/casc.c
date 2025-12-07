#include <pthread.h>
#include <stdio.h>
#include <assert.h>
#include <stdint.h>

extern intptr_t compare_and_swap(volatile intptr_t *p, intptr_t expected, intptr_t newv);
static volatile intptr_t val = 0;
const int MAX_NUM = 100;
const int MAX_THREAD_LIMIT = 15;

void *worker(void *args) {
    for (int i = 0; i < MAX_NUM; i++){
        int new, old;
        do{
            old = val;
            new = old + 1;
        }
        while(compare_and_swap(&val, old, new) != 0);
        
    }
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

    printf("val = %ld\n", val);
    return 0;
}