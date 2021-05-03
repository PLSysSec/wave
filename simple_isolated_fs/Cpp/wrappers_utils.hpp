#include <linux/limits.h>
#include <stdbool.h>
#include <queue>
#include <map>

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



// host -> sandbox
std::map<int, int> fd_map { };

// sandbox -> host
std::map<int, int> fd_rev_map { };

// std::set<int> free_fds { } ;
std::priority_queue<int> free_fds;

int fd_counter = 0;

int create_seal(int host_fd){
    int sandbox_fd;
    if (free_fds.empty()){
        sandbox_fd = fd_counter++;
    }
    else{
        sandbox_fd = free_fds.top();
        free_fds.pop();
    }

    fd_map[host_fd] = sandbox_fd;
    fd_rev_map[sandbox_fd] = host_fd;
    return sandbox_fd;
}

void remove_seal(int sandbox_fd){
    fd_rev_map.erase(sandbox_fd);
    fd_map.erase(fd_rev_map[sandbox_fd]);
    free_fds.push(sandbox_fd);
}

int sealed_to_sandbox(int host_fd)
{
    if (fd_map.find(host_fd) != fd_map.end()){
        return fd_map[host_fd];
    }
    else{
        return create_seal(host_fd);
    }
}

int sealed_from_sandbox(int sandbox_fd)
{
    int host_fd = fd_rev_map[sandbox_fd];
    return host_fd;
}
