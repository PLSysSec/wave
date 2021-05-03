Policy(read) = { fd = FdType, buf = SizedBuf(count) }
ssize_t read(int fd, void *buf, size_t count); 

Policy(write) = { fd = FdType, buf = SizedBuf(count) }
ssize_t write(int fd, const void *buf, size_t count); 

Policy(open) =  { pathname = PathType }
int open(const char *pathname, int flags); 

Policy(close) = { fd = FdType }
int close(int fd); 
