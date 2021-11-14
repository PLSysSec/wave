#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>

// create a link to data/tmp.txt and then cat the link
int main() {
   // 1. create link
   link("./data/tmp.txt", "./data/link.txt");

   // 2. open file
   int fd = open("./data/link.txt", O_RDONLY);
   int new_fd = 7;
   ssize_t count;

   char buffer[1024];


   // 3. cat file
   while ((count = read(fd, buffer, sizeof(buffer))) != 0)
      write(1, buffer, count); //stdout


   printf("Done!\n");
   return 0;
}
