#include <sys/time.h>
#include <stdio.h>

// [[{type:"module", name:"count_to"}]]

void count_to(int num){
    // [[{type:"event", name:"count_to::init"}]]
    for (int i = 1; i<=num; i++){
        // [[{type:"event", name:"count_to::print"}]]
        printf("i = %d\n", i);
    }
}


int main() {
    printf("Started Program\n");
    // [[{type:"event", name:"::count to 10"}]]
    printf("Counting to 10\n");
    count_to(10);
    // [[{type:"event", name:"::count to 10"}]]
    printf("Counting to 10\n");
    count_to(10);
    // [[{type:"event", name:"::count to 7"}]]
    printf("Counting to 7\n");
    count_to(7);
    printf("Finished Program\n");
}
