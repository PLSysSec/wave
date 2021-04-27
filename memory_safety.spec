Policy(Read) = { buf = SizedBuf(count) }
ssize_t read(int fd, void *buf, size_t count); 

Policy(Write) = { buf = SizedBuf(count) }
ssize_t write(int fd, const void *buf, size_t count); 

Policy(Open) =  { pathname = PathType }
int open(const char *pathname, int flags); 

Policy(Close) = {  }
int close(int fd); 
