#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <sys/mman.h>

const char* instructions = "\x48\x31\xFF\xB8\x3C\x00\x00\x00\x0F\x05";

int main() {
    printf("        main @ %p\n", &main);
    printf("instructions @ %p\n", instructions);

    printf("making instuctions executable...\n");
    void* addr = (void*)((size_t)instructions & ~0xfff);
    int ret = mprotect(addr, 0x1000, PROT_READ | PROT_EXEC);
    if (ret != 0) {
        printf("mprotect failed: error %d\n", errno);
        return 1;
    }

    printf("jumping...\n");
    void (*f)(void) = (void*)instructions;
    f();
    printf("after jump\n");
}
