#include <stdio.h>
#include <stdlib.h>

const int c = 29;

int main() {
    int n = 13;
    void* p = malloc(sizeof(int));
    printf("main is at %p\n", &main);
    printf("static is at %p\n", &c);
    printf("heap is at %p\n", p);
    printf("stack is at %p\n", &n);
    free(p);
}
