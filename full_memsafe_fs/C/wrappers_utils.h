#include <linux/limits.h>
#include <stdbool.h>

typedef char* hostptr;
typedef unsigned int sandboxptr;


unsigned long long membase = 0x40000000;
unsigned long long memlen =  0x10000000;

bool empty_predicate(){
    return true;
}

sandboxptr ptr_to_sandbox(hostptr buf)
{
    return buf - (hostptr)membase;
}


hostptr ptr_from_sandbox(sandboxptr buf)
{
    return buf + (hostptr)membase;
}


// sandboxptrs 
bool inBounds(sandboxptr ptr){
    return (ptr >= membase) && (ptr <= (membase + memlen));  
}

sandboxptr sized_buf_to_sandbox(hostptr buf, size_t size)
{
    return ptr_to_sandbox(buf);
}

// returns pointer if success, or null if memory violation
hostptr sized_buf_from_sandbox(sandboxptr buf, size_t size)
{
    if ((size < memlen) && inBounds(membase + buf) && inBounds(membase + buf + size)){
        return ptr_from_sandbox(buf);
    }
    else{
        return NULL;
    }
}


sandboxptr path_to_sandbox(hostptr buf)
{
    return sized_buf_to_sandbox(buf, PATH_MAX);
}


hostptr path_from_sandbox(sandboxptr buf)
{
    return sized_buf_from_sandbox(buf, PATH_MAX);
}


// inline int sealed_to_sandbox()
// {
//     return 2;
// }


// inline int sealed_from_sandbox()
// {
//     return 2;
// }
