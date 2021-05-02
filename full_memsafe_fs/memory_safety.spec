Policy(read) = { buf = SizedBuf(count) }
ssize_t read(int fd, void *buf, size_t count); 

Policy(write) = { buf = SizedBuf(count) }
ssize_t write(int fd, const void *buf, size_t count); 

Policy(open) =  { pathname = PathType }
int open(const char *pathname, int flags); 

Policy(close) = {  }
int close(int fd); 

Policy(fstat) = {  }
int fstat(int fd, struct stat *statbuf);

Policy(lseek) = {  }
off_t lseek(int fd, off_t offset, int whence);

Policy(dup2) = {  }
int dup2(int oldfd, int newfd);

Policy(unlink) = { pathname = PathType }
int unlink(const char *pathname);

Policy(symlink) = { target = PathType, linkpath = PathType }
int symlink(const char *target, const char *linkpath);

Policy(readlink) = { pathname = Pathtype, buf = SizedBuf(bufsiz) }
ssize_t readlink(const char *pathname, char *buf, size_t bufsiz);

Policy(getcwd) = { buf = SizedBuf(size) }
char *getcwd(char *buf, size_t size);

Policy(chdir) = { path = PathType }
int chdir(const char *path);

Policy(mkdir) = { pathname = PathType }
int mkdir(const char *pathname, mode_t mode);

Policy(rmdir) = { pathname = PathType }
int rmdir(const char *pathname);

