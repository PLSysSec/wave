#include <stdio.h>
#include <unistd.h>

int main() {
   printf("Please wait for 5 seconds...\n");
   sleep(5);
   printf("Thank you for waiting!\n");
   return 0;
}
