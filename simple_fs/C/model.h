#include "smack.h"
#include <stdlib.h>
#include <stdbool.h>
#include <string.h>
#include <errno.h>
#include <fcntl.h>

// Linux model of system calls
// should be oblivious to any sandboxing details

#define MAX_FILES 1024
#define MAX_FDS 1024

typedef struct fd_entry{
    unsigned inum;
    unsigned cursor;
    unsigned permissions;
} fd_entry;

typedef struct inode{
    int filesize;
    char* file;
} inode;

bool* inode_exists;
inode* inodes;
bool* fd_open; 
fd_entry* fdtable;

bool init_model(){
    inode_exists = calloc(1024, sizeof(bool));
    inodes = calloc(1024, sizeof(inode));
    fd_open = calloc(1024, sizeof(bool));
    fdtable = calloc(1024, sizeof(fd_entry));
    return (inode_exists && inodes && fd_open && fdtable);    
}

void cleanup_model(){
    free(inode_exists);
    free(inodes);
    free(fd_open);
    free(fdtable);
}

ssize_t model_read(int fd, void *buf, size_t count){
     if (fd < 0 || fd >= 1024 || !fd_open[fd]){
        errno = EBADF;
        return -1; // EBADF
    }
    fd_entry fdentry = fdtable[fd];
    unsigned cur = fdentry.cursor;
    inode inode_entry = inodes[fdentry.inum];
    int filesize = inode_entry.filesize;
    if (cur + count >= filesize){
        memcpy(buf, inode_entry.file + cur, filesize - cur - 1);
        fdtable[fd].cursor = filesize - 1;
        return filesize - cur - 1;
    }
    else{
        memcpy(inode_entry.file + cur, buf, count);
        fdtable[fd].cursor += count;
        return count;
    }
}

ssize_t model_write(int fd, const void *buf, size_t count){
    if  (fd < 0 || fd >= 1024 || !fd_open[fd]){
        errno = EBADF;
        return -1; // EBADF
    }
    fd_entry fdentry = fdtable[fd];
    unsigned cur = fdentry.cursor;
    inode inode_entry = inodes[fdentry.inum];
    int filesize = inode_entry.filesize;
    if (cur + count >= filesize){
        memcpy(inode_entry.file + cur, buf, filesize - cur - 1);
        fdtable[fd].cursor = filesize - 1;
        return filesize - cur - 1;
    }
    else{
        memcpy(buf, inode_entry.file + cur, count);
        fdtable[fd].cursor += count;
        return count;
    }
}

int model_open(const char *pathname, int flags){
    //TODO: add an assert here for allowed flags?
    //TODO: should we actually track what paths exist?
    //CREATE, TRUNC, EXCL
    bool path_exists = __VERIFIER_nondet_int();
    unsigned allocated_inum = __VERIFIER_nondet_unsigned();
    assume(allocated_inum < 1024);
    assume(inode_exists[allocated_inum] == false);
    if ((flags & O_CREAT) == O_CREAT){
        if (!path_exists){
            inode_exists[allocated_inum] = true;
            //TODO: what should this point to?
            inode new_inode = {0, NULL};
            inodes[allocated_inum] = new_inode; 
            path_exists = true;
        }
        else{
            if ((flags & O_EXCL) == O_EXCL){
                errno = EEXIST;
                return -1;
            }
        }
    }
    //Path should definitely exist at this point
    if (!path_exists){
        errno = ENOENT;
        return -1;
    }

     if ((flags & O_TRUNC) == O_TRUNC){
         inodes[allocated_inum].filesize = 0;
     }

    //currently all we assume is that fd has not already been allocated
    // as in a legal range
    // no assumption about getting the lowest fd
    unsigned fd = __VERIFIER_nondet_unsigned();
    assume(fd > 0 && fd < 1024);
    assume(fd_open[fd] == false);

    fd_open[fd] = true;
    fd_entry fdentry = {allocated_inum, 0, O_RDONLY | O_WRONLY | O_RDWR};
    fdtable[fd] = fdentry;
    return fd;

}

int model_close(int fd){
    if (fd < 0 || fd >= 1024 || !fd_open[fd]){
        errno = EBADF;
        return -1; // EBADF
    }
    fd_open[fd] = false;
    return 0;
    
} 
