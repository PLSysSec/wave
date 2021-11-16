#include <stdio.h>
#include <fcntl.h>


int main() {
   int fd = open("./data/tmp.txt", O_RDONLY);

   int oldflags = fcntl (fd, F_GETFL, 0);
   /* If reading the flags failed, return error indication now. */
   if (oldflags == -1)
     return -1;
   printf("oldflags = %x\n", oldflags);
   /* Set just the flag we want to set. */
   oldflags |= O_APPEND;
   /* Store modified flag word in the descriptor. */
   fcntl (fd, F_SETFL, oldflags);
   
   int newflags = fcntl (fd, F_GETFL, 0);
   printf("newflags = %x\n", newflags);
   printf("Done!\n");
   return 0;
}
