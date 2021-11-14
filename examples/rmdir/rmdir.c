#include <stdio.h>
#include <unistd.h>

int main() {
   rmdir("./remove_me");
   printf("Done!!\n");
   return 0;
}
