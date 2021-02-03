//
// Created by wyy on 2021/1/11.
//

#include "./network_traffic.h"
#include <stdio.h>
#include <stdbool.h>

bool IsInHeap(const void * pObj)
{
    int n;
    if ( pObj < &n )
    {
        return true;
    }
    return false;
}

void data(ProcessStatistics item) {
    printf("item in heap %d, list in heap %d\n", IsInHeap(&item), IsInHeap(item.list));
    printf("length: %lu\n", item.length);
    fflush(stdout);
    for (int i = 0; i < item.length; i++) {
        printf("pid: %d, download: %lu, upload: %lu\n", item.list[i].pid, item.list[i].download_length, item.list[i].upload_length);
        fflush(stdout);
    }
    free_array(item);
//    free(item.list);
}

int main() {
    printf("Hello world\n");
    take(data);
    return 0;
}

//#gcc test.c -o test -I./network_traffic.h ./target/debug/libnetwork_traffic.dylib && \
//LD_LIBRARY_PATH=./target/debug ./test
