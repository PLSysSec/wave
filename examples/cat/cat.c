#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <errno.h>
 
int main(int argc, char *argv[]) {
  FILE *file;
  int chr;
  /*
  int count;
 
  for(count = 1; count < argc; count++) {
    if((file = fopen(argv[count], "r")) == NULL) {
      fprintf(stderr, "%s: %s : %s\n", argv[0], argv[count],
        strerror(errno));
    continue;
    }
    while((chr = getc(file)) != EOF)
      fprintf(stdout, "%c", chr);
    fclose(file);
  }*/

    // For now, just hardcode the path
    file = fopen("./data/tmp.txt", "r");
    if (file == NULL) {
    	printf("fopen result: %s\n", strerror(errno));
    }
    //printf("file = %d", file);
    while((chr = getc(file)) != EOF)
      fprintf(stdout, "%c", chr);
    fclose(file);

  exit(0);
}

