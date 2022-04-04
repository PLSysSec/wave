#include <stdio.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>
#include <errno.h>
#include <string.h>
#include <time.h>

int main(int argc, char *argv[]) {
  clock_t start = clock();
  clock_t end = clock();
  printf("time = %lld\n", end - start);
  // __imported_wasi_snapshot_preview1_clock_res_get();
  struct timespec res_get;
  int result = clock_getres(CLOCK_PROCESS_CPUTIME_ID, &res_get);
  printf("result = %d res_get.tv_sec = %lld res_get.tv_nsec = %ld\n", result, res_get.tv_sec, res_get.tv_nsec);

//   struct stat st = {0};
//   if (stat("test_dir", &st) == -1) {
//     printf("mkdir result = %d %d %s\n",mkdir("test_dir", 0700), errno, strerror(errno));
// }
  printf("Done!\n");
}


