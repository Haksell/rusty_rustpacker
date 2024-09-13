#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <sys/mman.h>

const char* instructions = "\x48\x31\xFF\xB8\x3C\x00\x00\x00\x0F\x05";
const size_t instructions_len = 10;

int main() {
    printf("        main @ %p\n", &main);
    printf("instructions @ %p\n", instructions);

    size_t region = (size_t)instructions;
    region &= ~0xfff;
    printf("        page @ %lx\n", region);

    printf("making page executable...\n");
    int ret = mprotect((void*)region, 0x1000, PROT_READ | PROT_EXEC);
    if (ret != 0) {
        printf("mprotect failed: error %d\n", errno);
        return 1;
    }

    void (*f)(void) = (void*)instructions;
    printf("jumping...\n");
    f();
    printf("after jump\b");
}
