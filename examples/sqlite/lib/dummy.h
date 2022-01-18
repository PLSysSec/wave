#ifndef DUMMY_FOR_WASI
#define DUMMY_FOR_WASI

#include <stdlib.h>
#include <sys/types.h>
#include <stdio.h>

int fchmod(int fildes, mode_t mode) {
    fprintf(stderr, "\nsearchme Calling fchmod stub\n\n");
    return 1;
}

int fchown(int fildes, uid_t owner, gid_t group) {
    fprintf(stderr, "\nsearchme Calling fchown stub\n\n");
    abort();
}

uid_t geteuid(void) {
    fprintf(stderr, "\nsearchme Calling geteuid stub\n\n");
    return 42;
}

int getpagesize(void) {
    fprintf(stderr, "\nsearchme Calling getpagesize stub\n\n");
    return 4096;
}

#define F_RDLCK 0
#define F_WRLCK 1
#define F_UNLCK 2
#define F_GETLK 5
#define F_SETLK 6

#endif