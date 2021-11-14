#include <stdio.h>

extern char **environ;


int main(int argc, char* argv[]){
   //1. print commandline args
   for(int counter=0;counter<argc;counter++){
      printf("\nargv[%d]: %s",counter,argv[counter]);
      printf("\nNumber Of Arguments Passed: %d",argc);
      printf("\n----Following Are The Command Line Arguments Passed----");
   }

   //2. print environment variables
   while (*environ){
      printf("Environment Variable: %s\n", *environ++);
   }

   printf("Done!\n");
   return 0;
}
