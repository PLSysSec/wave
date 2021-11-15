#include <stdio.h>
#include <sys/types.h> 
#include <sys/stat.h> 
#include <unistd.h> 
#include <errno.h> 
#include <string.h> 
#include <fcntl.h> 
#include <time.h>


// struct stat { 
//                dev_t     st_dev;     /* ID of device containing file */ 
//                ino_t     st_ino;     /* inode number */ 
//                mode_t    st_mode;    /* protection */ 
//                nlink_t   st_nlink;   /* number of hard links */ 
//                uid_t     st_uid;     /* user ID of owner */ 
//                gid_t     st_gid;     /* group ID of owner */ 
//                dev_t     st_rdev;    /* device ID (if special file) */ 

//                off_t     st_size;    /* total size, in bytes */ 
//                blksize_t st_blksize; /* blocksize for file system I/O */ 
//                blkcnt_t  st_blocks;  /* number of 512B blocks allocated */ 

//                time_t    st_atime;   /* time of last access */ 
//                time_t    st_mtime;   /* time of last modification */ 
//                time_t    st_ctime;   /* time of last status change */ 
//            };


void pprint_stat(struct stat st){
   printf("Device containing file :  %llu\n", st.st_dev);
   printf("Inode # :  %llu\n", st.st_ino);
   printf("Permissions # :  %d\n", st.st_mode);
   printf("# of hard links :  %llu\n", st.st_nlink);
   printf("UID :  %d\n", st.st_uid);
   printf("GID :  %d\n", st.st_gid);
   printf("Device ID (if special file) :  %llu\n", st.st_rdev);
   printf("Size :  %lld\n", st.st_size);
   printf("Block size :  %ld\n", st.st_blksize);
   printf("Block count :  %lld\n", st.st_blocks);
   printf("Last Access was :  %s\n",ctime(&st.st_atime));
   printf("Last Modification was :  %s\n",ctime(&st.st_mtime));
   printf("Last Status change was :  %s\n",ctime(&st.st_ctime));
}

int main() {
   int fd = open("./data/tmp.txt", O_RDONLY);
   struct stat st;

   // 1. Do initial stat
   if(fstat(fd, &st)){ 
        printf("\nfstat error: [%s]\n",strerror(errno)); 
        close(fd); 
        return -1; 
    } 

   // printf("Device containing file :  %llu\n", st.st_dev);
   // printf("Inode # :  %llu\n", st.st_ino);
   // printf("Permissions # :  %d\n", st.st_mode);
   // printf("# of hard links :  %llu\n", st.st_nlink);
   // printf("UID :  %d\n", st.st_uid);
   // printf("GID :  %d\n", st.st_gid);
   // printf("Device ID (if special file) :  %llu\n", st.st_rdev);
   // printf("Size :  %lld\n", st.st_size);
   // printf("Block size :  %ld\n", st.st_blksize);
   // printf("Block count :  %lld\n", st.st_blocks);
   // printf("Last Access was :  %s\n",ctime(&st.st_atime));
   // printf("Last Modification was :  %s\n",ctime(&st.st_mtime));
   // printf("Last Status change was :  %s\n",ctime(&st.st_ctime));
   pprint_stat(st);

   // 2. Extend file by 1 byte
   off_t previous_size = st.st_size;
   ftruncate(fd, previous_size + 1);

   if(fstat(fd, &st)){ 
        printf("\nfstat error: [%s]\n",strerror(errno)); 
        close(fd); 
        return -1; 
    } 

   pprint_stat(st);

   printf("Done!\n");


   // 3. Update timestamp 
   //    - first my resetting to start of epoch with futimens
   //    - then by resetting to current time with utimensat
   /*
      int utimensat(int dirfd, const char *pathname, const struct timespec times[2], int flags);
      int futimens(int fd, const struct timespec times[2]);
   */

   struct timespec atim = {0, 0};
   struct timespec mtim = {0, 0};
   struct timespec times[2] = {atim, mtim};

   futimens(fd, times);
   
   if(fstat(fd, &st)){ 
        printf("\nfstat error: [%s]\n",strerror(errno)); 
        close(fd); 
        return -1; 
    } 

   pprint_stat(st);


   utimensat(3, "./data/tmp.txt", NULL, 0);
   
   if(fstat(fd, &st)){ 
        printf("\nfstat error: [%s]\n",strerror(errno)); 
        close(fd); 
        return -1; 
    } 

   pprint_stat(st);

   return 0;
}
