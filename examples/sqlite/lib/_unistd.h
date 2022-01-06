#ifndef UNISTD_FOR_WASI
#define UNISTD_FOR_WASI

#include <stdlib.h>

int fchown(int fildes, uid_t owner, gid_t group) {
    abort();
}

uid_t geteuid(void) {
    abort();
}

#endif