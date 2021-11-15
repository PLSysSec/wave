#include <stdio.h>
#include <unistd.h>
#include <fcntl.h>


// These three calls are all unobservable (or close to unobservable)
// Really I just expect all these calls to succeed
int main() {
   int fd = open("./data/tmp.txt", O_RDONLY);
   fsync(fd);
   fdatasync(fd);
   posix_fadvise(fd, 0, 10, POSIX_FADV_SEQUENTIAL);
   close(fd);
   printf("Done!\n");
   return 0;
}
