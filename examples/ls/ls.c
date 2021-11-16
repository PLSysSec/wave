#include <stdio.h>
#include "string.h"
#include <dirent.h>

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
    }
    printf("\n");
}

int main() {
   do_ls(".");
   printf("Done!\n");
   return 0;
}
