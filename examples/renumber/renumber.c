#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>


int main() {
   // 1. open file
   int fd = open("./data/tmp.txt", O_RDONLY);
   int new_fd = 7;
   ssize_t count;

   char buffer[1024];

   // 2. renumber file
   __wasi_fd_renumber(fd, new_fd);

 // 3. cat file
 while ((count = read(new_fd, buffer, sizeof(buffer))) != 0)
      write(1, buffer, count); //stdout


   printf("Done!\n");
   return 0;
}
