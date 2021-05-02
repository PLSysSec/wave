Policy(read) = { buf = SizedBuf(count) }
ssize_t read(int fd, void *buf, size_t count); 

Policy(write) = { buf = SizedBuf(count) }
ssize_t write(int fd, const void *buf, size_t count); 

Policy(open) =  { pathname = PathType }
int open(const char *pathname, int flags); 

Policy(close) = {  }
int close(int fd); 
