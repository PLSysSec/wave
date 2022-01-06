#ifndef PTHREAD_FOR_WASI
#define PTHREAD_FOR_WASI

#define __DEFINED_pthread_t
#define __DEFINED_pthread_attr_t
#define __DEFINED_pthread_mutex_t
#define __DEFINED_pthread_mutexattr_t

// #include <bits/alltypes.h>

typedef struct __pthread * pthread_t;
typedef struct { union { int __i[sizeof(long)==8?14:9]; volatile int __vi[sizeof(long)==8?14:9]; unsigned long __s[sizeof(long)==8?7:9]; } __u; } pthread_attr_t;
typedef struct { union { int __i[sizeof(long)==8?10:6]; volatile int __vi[sizeof(long)==8?10:6]; volatile void *volatile __p[sizeof(long)==8?5:6]; } __u; } pthread_mutex_t;
typedef struct { unsigned __attr; } pthread_mutexattr_t;

#define PTHREAD_MUTEX_INITIALIZER 0
#define PTHREAD_MUTEX_RECURSIVE 0

int pthread_create(pthread_t *restrict thread,
                   const pthread_attr_t *restrict attr,
                   void *(*start_routine)(void *),
                   void *restrict arg) {
    return 1; // indicate error
}

int pthread_join(pthread_t thread, void **retval) {
    return 0; // should not be reachable
}

pthread_t pthread_self(void) {
    return (pthread_t) 0;
}

int pthread_equal(pthread_t t1, pthread_t t2) {
    return t1 == t2;
}


int pthread_mutex_init(pthread_mutex_t *restrict mutex,
                       const pthread_mutexattr_t *restrict attr) {
    return 0;
}

int pthread_mutex_destroy(pthread_mutex_t *mutex) {
    return 0;
}

int pthread_mutex_lock(pthread_mutex_t *mutex) {
    return 0;
}

int pthread_mutex_trylock(pthread_mutex_t *mutex) {
    return 0;
}

int pthread_mutex_unlock(pthread_mutex_t *mutex) {
    return 0;
}

int pthread_mutexattr_init(pthread_mutexattr_t *attr) {
    return 0;
}

int pthread_mutexattr_destroy(pthread_mutexattr_t *attr) {
    return 0;
}
int pthread_mutexattr_settype(pthread_mutexattr_t *attr, int type) {
    return 0;
}

#endif
