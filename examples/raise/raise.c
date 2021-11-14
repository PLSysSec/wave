#include <stdio.h>
//#include <signal.h>
//int raise(int sig);

// This test doesn't really do much, all I'm expecting is that it doesn't crash
int main() {
   __wasi_proc_raise(11);
   printf("Done!\n");
   return 0;
}
