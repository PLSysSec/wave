#include <stdio.h>
#include <unistd.h>

// int symlink(const char *target, const char *linkpath);
// ssize_t readlink(const char *restrict pathname, char *restrict buf, size_t bufsiz);


int main() {
   symlink("./data/tmp.txt","./data/link");
   char buf[1024];
   readlink("./data/link", &buf, 1024);
   printf("Contents of symlink = %s\n", buf);
   symlink("./data/tmp.txt","./data/link2");
   unlink("./data/link2");
   printf("Done!\n");
   return 0;
}
