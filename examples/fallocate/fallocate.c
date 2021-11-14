#include <stdio.h>
#include <fcntl.h>

int main() {
   //printf("Hello, World!\n");
   //file = fopen("./data/tmp.txt", "r");
   int fd = open("./output/tmp.txt", O_WRONLY | O_CREAT | S_IRUSR | S_IWUSR);

   //int fallocate(int fd, int mode, off_t offset, off_t len);
   // create an empty file of size 100 bytes
   posix_fallocate(fd, 0, 100);
   printf("Done!\n");
   return 0;
}
