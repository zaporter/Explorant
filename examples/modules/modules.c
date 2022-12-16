#include <stdio.h>
// [[{type:"module", name:"util"}]]
// [[{type:"module", name:"print", parent_module:"util"}]]
// [[{type:"module", name:"add", parent_module:"util"}]]

int increment(int a){
    // Define a start event that gets expanded into ::util::add::entry
    // [[{type:"event", name:"add::entry"}]]
    a = a+1;
    return a;
}
void print_num(int a){
    // Define a start event that gets expanded into ::util::print::entry
    // [[{type:"event", name:"print::entry"}]]
    printf("val: %d\n", a);
}

int main(){
    // [[{type:"event", name:"::entry"}]]
    int a = increment(2);
    print_num(a);
    // [[{type:"event", name:"::exiting after printing the num"}]]
    return 0;
}

