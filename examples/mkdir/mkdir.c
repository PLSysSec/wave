#include <stdio.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>




int main(int argc, char *argv[]) {
  struct stat st = {0};
  if (stat("test_dir", &st) == -1) {
    mkdir("test_dir", 0700);
}
  printf("Done!\n");
}

