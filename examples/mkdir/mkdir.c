#include <stdio.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>
#include <errno.h>
#include <string.h>


int main(int argc, char *argv[]) {
  struct stat st = {0};
  if (stat("test_dir", &st) == -1) {
    printf("mkdir result = %d %d %s\n",mkdir("test_dir", 0700), errno, strerror(errno));
}
  printf("Done!\n");
}

