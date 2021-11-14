#include <stdio.h>

// int rename(const char *oldpath, const char *newpath);

int main() {
   rename("A.txt", "B.txt");
   printf("Done!\n");
   return 0;
}
