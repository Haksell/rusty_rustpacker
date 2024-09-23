#include <stdint.h>
#include <stdio.h>

const char* instructions = "\x48\x31\xFF\xB8\x3C\x00\x00\x00\x0F\x05";

int main() {
    printf("        main @ %p\n", &main);
    printf("instructions @ %p\n", instructions);
    void (*f)(void) = (void*)instructions;
    printf("jumping...\n");
    f();
    printf("after jump\n");
}
