#include <stdio.h>
#include "string.h"
#include <dirent.h>
#include <errno.h>

void do_ls(char* path)
{
  printf("I'm doing an ls(%s)!\n", path);

    DIR *d;
    struct dirent *dir;
    d = opendir(path);
    if (d)
    {
        while ((dir = readdir(d)) != NULL)
        {
            printf("%s    ", dir->d_name);
        }
        closedir(d);
    } else {
        printf("d not valid! errno: %d", errno);
    }
    printf("\n");
}

int main() {
   do_ls(".");
   printf("Done!\n");
   return 0;
}
