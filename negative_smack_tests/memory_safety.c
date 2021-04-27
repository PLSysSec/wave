#include <stdint.h>
#include <stdlib.h>

//fails
void* memory_leak()
{
    void* x = malloc(32);
    return x;
} 

//fails
void double_free_1(uint32_t* x)
{
    free(x);
} 

//fails
void double_free_2()
{
    void* x = malloc(32);
    free(x);
    free(x);
} 

//fails
uint32_t use_after_free()
{
    void* x = malloc(32);
    free(x);
    return *((uint32_t*)(x));
} 

//passes, very cool
void not_double_free()
{
    free(memory_leak());
} 

//fails
uint32_t invalid_deref(uint32_t* x)
{
    return *x;
} 

//fails
uint32_t array_overflow(uint32_t x)
{
    uint32_t a[10];
    return a[x];
} 
