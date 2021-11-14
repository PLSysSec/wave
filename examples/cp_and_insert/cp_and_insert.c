#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include <fcntl.h>

int main(int argc, char **argv)
{
    char buffer[1024];
    int files[2];
    ssize_t count;

    /* Check for insufficient parameters */
    //if (argc < 3)
    //    return -1;
    files[0] = open("./data/tmp.txt", O_RDONLY);
    if (files[0] == -1) /* Check if file opened */
        return -1;
    files[1] = open("./output/tmp.txt", O_RDONLY | O_WRONLY | O_CREAT | S_IRUSR | S_IWUSR);
    if (files[1] == -1) /* Check if file opened (permissions problems ...) */
    {
        close(files[0]);
        return -1;
    }

    while ((count = read(files[0], buffer, sizeof(buffer))) != 0)
        write(files[1], buffer, count);

    // will invoke fd_tell (all lseeks of the form lseek(x,0,SEEK_CUR) do)
    int position = lseek(files[1], 0, SEEK_CUR); 
    printf("position for lseek1 = %d\n", position);

    lseek(files[1], 0, SEEK_SET); //get back to the beginning

    position = lseek(files[1], 0, SEEK_CUR); 
    printf("position for lseek2 = %d\n", position);

    // Change
    // This is the contents of the file!
    // to
    // This is the contents in the file!
    pwrite(files[1], "in", 2, 21);
       
    // read the word file from ./output/tmp.txt   
    char buf[4];
    pread(files[1], buf, 4, 28);
    printf("results of pread = %s\n", buf); 


    printf("Done!\n");
    return 0;
}

