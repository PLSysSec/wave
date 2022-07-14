#include <stdio.h>
#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <fcntl.h>

int main() {
    unsigned char buf[] = {2,200,0,0,0,0,0,0,0,200,0,0,0,0,0,0,0,0x84,0x3,0,0,0,0,0,0,0x90,0x1,0,0,0,0,0,0};
    FILE* ptr = fopen("./new_test.txt", "w+");
    fwrite(buf, 1, 45, ptr);
    fclose(ptr);
}
