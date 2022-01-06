#ifndef STAT_FOR_WASI
#define STAT_FOR_WASI

#include <stdlib.h>

int fchmod(int fildes, mode_t mode) {
    abort();
}

#endif