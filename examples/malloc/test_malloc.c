#define _GNU_SOURCE
#include <assert.h>
#include <gnu/libc-version.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>

// [[{type:"module", name:"mmap"}]]
// [[{type:"module", name:"bins"}]]
// [[{type:"module", name:"glibc"}]]
// [[{type:"module", name:"malloc", parent_module:"glibc"}]]
// [[{type:"module", name:"free", parent_module:"glibc"}]]
int main(int argc, char **argv) {
    // [[{type:"event", name:"::print glibc version"}]]
    printf("gnu_get_libc_version() = %s\n", gnu_get_libc_version());

    // [[{type:"event", name:"mmap::malloc large chunk"}]]
    int* k = malloc(2048*2048);
    k[0]= 10;
    // [[{type:"event", name:"::Print k"}]]
    printf("k[0] is: %d\n", k[0]);
    // [[{type:"event", name:"mmap::free k"}]]
    free(k);
        
    // [[{type:"event", name:"bins::entry"}]]
    for (int i = 2050; i>200; i-=3) {
        // [[{type:"event", name:"bins::loop"}]]
        printf("i is: %d\n", i);
        int j=i%113;
        // [[{type:"event", name:"bins::malloc"}]]
        int* k = malloc(j*sizeof(int));
        if (i%4==0){
            // [[{type:"event", name:"bins::free"}]]
            free(k);
        }
    }
}
