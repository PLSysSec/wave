#include <stdio.h>
#include <unistd.h>


     
// This test just prints 8 random hex butes
int main(void){
   int length = 8;
   char str[] = "0123456789ABCDEF";

   unsigned char buf[8];
   unsigned char outbuf[9];
   getentropy(buf, 8);

   // translate to hex
   for (int i = 0; i < 8; i++){
      int index = buf[i] % 16;
      outbuf[i] = str[buf[i] % 16];
   }
   outbuf[8] = '\0';
   printf("outbuf = %s\n", outbuf);
   printf("Done!\n");
     
}