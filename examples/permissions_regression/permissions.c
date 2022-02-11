#include <fcntl.h>
#include <unistd.h>
#include <stdio.h>

int main() {
   printf("Hello, World!\n");
   int fd = open("log.txt", O_RDWR | O_CREAT);
   char buf[] = "hello hello";
   int ret = write(fd, buf, 11);
   printf("write returns %d\n", ret);
   close(fd);

   FILE *fp = fopen("not_exist.txt", "w");
   fwrite(buf, 1, 11, fp);

   return 0;
}

