#include <stdint.h>

//passes
uint32_t overflow_u32(uint32_t x)
{
    return x + 2;
} 

//fails
int32_t overflow_s32(int32_t x)
{
    return x + 2;
} 

//passes
uint64_t overflow_u64(uint64_t x)
{
    return x + 2;
} 

//fails
int64_t overflow_s64(int64_t x)
{
    return x + 2;
} 

//passes
void* overflow_ptr(void* x)
{
    return x + 2;
} 

//passes
uint64_t underflow_u64(uint64_t x)
{
    return x - 2;
}

//fails
int64_t underflow_s64(int64_t x)
{
    return x - 2;
}

//passes
void* underflow_ptr(void* x)
{
    return x - 2;
}
