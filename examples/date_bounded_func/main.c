#include <sys/time.h>
#include <stdio.h>

// [[{type:"module", name:"divisible"}]]

long long current_epoch() {
    // [[{type:"event", name:"::get the current time"}]]
    struct timeval te; 
    gettimeofday(&te, NULL); 
    return te.tv_sec;
}

void run(){
    // [[{type:"event", name:"divisible::enter"}]]
    for (int i=0;i<10; i++){
        // [[{type:"event", name:"divisible::loop and print"}]]
        printf("Current time is divisible by 2!\n");
    }
    
}
int main() {
    // [[{type:"event", name:"::Start the program"}]]
    printf("Started Program\n");
    long long current_time = current_epoch();
    printf("Current time: %lld\n", current_time);
    // [[{type:"flow", name:"::Is the current time is divisible by 2?"}]]
    if (current_time%2 == 0){
        run();
    }else {
    // [[{type:"flow", name:"::It is not divisible by 2"}]]
        printf("Current time is not divisible by 2\n");
    }
    // [[{type:"event", name:"::Program finished"}]]
    printf("Finished Program\n");

}
