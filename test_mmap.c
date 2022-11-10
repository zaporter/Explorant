#define _GNU_SOURCE
#include <assert.h>
#include <gnu/libc-version.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char **argv) {
    printf("gnu_get_libc_version() = %s\n", gnu_get_libc_version());
    // PLACEHOLDER_KEY_1
    int* k = malloc(2048*2048);
    // PLACEHOLDER_KEY_2
    k[0]= 10;
    printf("k[0] is: %d\n", k[0]);
    // PLACEHOLDER_KEY_3
    free(k);
}
