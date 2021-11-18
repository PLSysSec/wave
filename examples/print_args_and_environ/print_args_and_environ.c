#include <stdio.h>

extern char **environ;


int main(int argc, char* argv[]){
   //1. print commandline args
  printf("Number of arguments passed: %d\n", argc); 

  for(int counter=0;counter<argc;counter++){
      printf("argv[%d]: %s\n",counter,argv[counter]);
   }


   //2. print environment variables
   while (*environ){
      printf("Environment Variable: %s\n", *environ++);
   }

   printf("Done!\n");
   return 0;
}
