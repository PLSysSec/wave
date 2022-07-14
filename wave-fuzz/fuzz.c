#include <asm/unistd.h>
#include <stdio.h>
#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <fcntl.h>
#include <sys/uio.h>
#include <sys/mman.h>
#include <stdlib.h>
#include <time.h>

#define IS_RET_ERROR(x) (x > -4096 && x < 0)

ssize_t raw_syscall(int nr, void* arg1, void* arg2, void* arg3) {
    ssize_t ret;
    asm volatile("syscall"
        : "=a" (ret)
        : "0"(nr), "D"(arg1), "S"(arg2), "d"(arg3)
        : "rcx", "r11", "memory"
        );
    return ret;
}

ssize_t raw_syscall3(int nr, void* arg1, void* arg2, void* arg3) {
    ssize_t ret;
    asm volatile("syscall"
        : "=a" (ret)
        : "0"(nr), "D"(arg1), "S"(arg2), "d"(arg3)
        : "r11", "memory"
        );
    return ret;
}

ssize_t raw_syscall4(int nr, void* arg1, void* arg2, void* arg3, void* arg4) {
    ssize_t ret;
    asm volatile("syscall"
        : "=a" (ret)
        : "0"(nr), "D"(arg1), "S"(arg2), "d"(arg3), "c" (arg4)
        : "r11", "memory"
        );
    return ret;
}


size_t page(void* addr) {
    return ((size_t)addr) & ~0xFFF;
}

size_t page_offset(void* addr) {
    return ((size_t)addr) & 0xFFF;
}

enum RegionType {
    TEXT,
    RODATA,
    DATA,
    DYLIB,
    STACK,
    HEAP,
    OTHER,
};

char* to_string(enum RegionType type) {
    switch (type) {
        case TEXT: return "text";
        case RODATA: return "rodata";
        case DATA: return "data";
        case DYLIB: return "dylib";
        case STACK: return "stack";
        case HEAP: return "heap";
        case OTHER: return "other";
        default: return "error";
    }
}

int contains(char* str, char* tail) {
    char* tail_ptr = tail;
    while (*str) {
        char* str_ptr = str;
        while (true) {
            if (!*tail_ptr) {
                return true;
            }

            if (!*str_ptr) {
                return false;
            }

            if (*str_ptr == *tail_ptr) {
                tail_ptr++;
                str_ptr++;
            } else {
                tail_ptr = tail;
                break;
            }
        }
        str++;
    }
    return false;
}

int str_to_prot(char* str) {
    int res = 0;
    if (str[0] == 'r') {
        res |= PROT_READ;
    }
    if (str[1] == 'w') {
        res |= PROT_WRITE;
    }
    if (str[2] == 'x') {
        res |= PROT_EXEC;
    }

    return res;
}

enum RegionType region_type_from_string_and_prot(char* str, char* bin_path, int prot) {
    bool label_is_bin_path = strcmp(str, bin_path) == 0;
    if(label_is_bin_path && prot & PROT_EXEC) {
        return TEXT;
    } else if (label_is_bin_path && (prot & PROT_EXEC) == 0 && (prot & PROT_WRITE) == 0) {
        return RODATA;
    } else if (label_is_bin_path && (prot & PROT_EXEC) == 0 && (prot & PROT_WRITE)) {
        return DATA;
    } else if (contains(str, ".so")) {
        return DYLIB;
    } else if (strcmp(str, "[stack]") == 0) {
        return STACK;
    } else if (strcmp(str, "[heap]") == 0) {
        return HEAP;
    }

    return OTHER;
}

struct MemoryRegion {
    void* addr;
    size_t len;
    enum RegionType type;
    int prot;
};

struct MemoryReader {
    struct MemoryRegion* regions;
    uint64_t regions_len;
    uint32_t regions_idx;
    uint32_t region_offset;
};

struct MemoryReader* new_reader(char* bin_path) {
    // TODO: remove memory leaks
    ssize_t addr = mmap(0xDEADB000, sizeof(struct MemoryReader) + sizeof(struct MemoryRegion) * 512 + 4096, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (addr == -1) {
        printf("Failed to mmap...exiting\n");
        exit(1);
    } else {
        printf("Got addr %x\n", addr);
        fflush(stdout);
    }
    struct MemoryReader* reader = (struct MemoryReader*)addr;
    addr += sizeof(struct MemoryReader);
    reader->regions = (struct MemoryRegion*)(addr);
    reader->regions_len = 0;
    reader->regions_idx = 0;
    reader->region_offset = 0;

    char* buf = (char*)malloc(256);
    size_t size = 256;
    sprintf(buf, "/proc/self/maps");
    FILE* maps = fopen(buf, "r");
    uint64_t start, end;
    char* region_info = (char*)calloc(256, 1);
    char* protections = (char*)malloc(64);
    char* offset = (char*) malloc(16);
    char* dev = (char*) malloc(16);
    char* inode = (char*) malloc(16);

    while (getline(&buf, &size, maps)) {
        int ret = sscanf(buf, "%lx-%lx %s %s %s %s %s", &start, &end, protections, offset, dev, inode, region_info);
        if (ret != 7) {
            int ret = sscanf(buf, "%lx-%lx %s %s", &start, &end, protections, region_info);
        }
        char* region_name = &region_info[256];
        int i = 256;
        // get the name of the region from the remaining info
        while ((*region_name != ' ' || *region_name == '\0') && region_name != region_info) {
            // remove trailing newline
            if (*region_name == '\n') {
                *region_name = '\0';
            }
            region_name--;
            i--;
        }
        // remove extra space in front of name
        region_name = region_name == region_info ? region_name : region_name++;

        if (reader->regions_idx != 0 && reader->regions[reader->regions_idx-1].addr == start) {
            break;
        }

        reader->regions[reader->regions_idx].addr = (void*)start;
        reader->regions[reader->regions_idx].len = end-start;
        reader->regions[reader->regions_idx].prot = str_to_prot(protections);
        reader->regions[reader->regions_idx].type = region_type_from_string_and_prot(region_name, bin_path, reader->regions[reader->regions_idx].prot);
        printf("%lx-%lx %s %s {%s}\n", start, end, protections, region_name, to_string(reader->regions[reader->regions_idx].type));
        reader->regions_idx++;
        reader->regions_len++;
        fflush(stdout);
    }

    return reader;
}

void protect_all(struct MemoryReader* reader, struct iovec* iovs, int num_iovs) {
    size_t regions_page = page(reader->regions);
    size_t regions_len = reader->regions_len + page_offset(reader->regions);
    size_t reader_page = page((size_t)reader);
    size_t reader_len = sizeof(struct MemoryReader) + page_offset(reader);

    size_t* safe_page = (size_t*)malloc(sizeof(size_t) * num_iovs);
    size_t* safe_plen = (size_t*)malloc(sizeof(size_t) * num_iovs);
    for (int i = 0; i < num_iovs; i++) {
        safe_page[i] = page(iovs[i].iov_base);
        safe_plen[i] = iovs[i].iov_len + page_offset(iovs[i].iov_base);
    }
    for (int i= 0; i < num_iovs; i++) {
        printf("addr: %lx, Safe addr and len: %lx %d\n", iovs[i].iov_base, safe_page[i], safe_plen[i]);
        fflush(stdout);
    }
    for (int i = 0; i < reader->regions_len; i++) {
        // for some reason, protecting these regions causes the binary to crash very often,
        // but only when AFL is running it. Just skip them for now
        if (i == 15 || i == 3 || i == 6 || i == 16 || i == 17) {
            continue;
        }
        //printf("region: i: %d, addr %lx, len: %lx, type: %lx...\n", i, reader->regions[i].addr, reader->regions[i].len, reader->regions[i].type == TEXT);
        if (reader->regions[i].type == STACK || reader->regions[i].type == TEXT || reader->regions[i].addr == reader || reader->regions[i].type == RODATA || reader->regions[i].type == DATA) {
           //printf("at continue: reader: %lx, addr: %lx, addr_len: %lx\n", reader, reader->regions[i].addr, reader->regions[i].len);
           //printf("done\n");
           continue;
        }
        int res = raw_syscall(__NR_mprotect, reader->regions[i].addr, reader->regions[i].len, PROT_NONE);
        // unprotect reader and regions so we can unprotect later correctly
        for (int j = 0; j < num_iovs; j++) {
            //printf("addr: %lx, Safe addr and len: %lx %d\n", iovs[j].iov_base, safe_page[j], safe_plen[j]);
            raw_syscall(__NR_mprotect, page(safe_page), num_iovs * sizeof(size_t) + page_offset(safe_page), PROT_READ | PROT_WRITE);
            raw_syscall(__NR_mprotect, page(safe_plen), num_iovs * sizeof(size_t) + page_offset(safe_plen), PROT_READ | PROT_WRITE);
            raw_syscall(__NR_mprotect, page(iovs), num_iovs * sizeof(struct iovec) + page_offset(iovs), PROT_READ | PROT_WRITE);
            raw_syscall(__NR_mprotect, safe_page[j], safe_plen[j], PROT_READ | PROT_WRITE);
        }
        //printf("done\n");
    }
}

void unprotect_all(struct MemoryReader* reader) {
    for (int i = 0; i < reader->regions_len; i++) {
        int res = raw_syscall(__NR_mprotect, reader->regions[i].addr, reader->regions[i].len, reader->regions[i].prot);
    }
}




void test_syscall(int syscall, int fd, const struct iovec *iov, int iovcnt, char* bin_path) {
    struct MemoryReader* reader = new_reader(bin_path);
    ssize_t result = 0;
    protect_all(reader, iov, iovcnt);
    //if (syscall == __NR_preadv || syscall == __NR_pwritev) {
    //    result = raw_syscall4(__NR_writev, fd, iov, iovcnt, 0);
    //} else {
    //    result = raw_syscall3(__NR_writev, fd, iov, iovcnt);
    //}
    unprotect_all(reader);
    printf("Got %d\n", result);
    fflush(stdout);
    if (result == -14) {
        abort();
    }
}

int main(int argc, char** argv) {
    if (argc < 2) {
        printf("Wrong number of args!\n");
        exit(1);
    }
    freopen("output.txt", "a+", stdout);
    char* resolved_path[256] = {0};
    realpath(argv[0], resolved_path);
    char* file_path = argv[1];
    char* input[1024];
    int input_offset = 0;
    int num_iovec = 0;
    FILE* ptr = fopen(file_path, "r");
    if (!ptr) {
        return 0;
    }
    num_iovec = (int)fgetc(ptr);

    if (num_iovec < 0 || num_iovec > 100) {
        return 0;
        //abort();
    }
    printf("num_iovec: %d\n", num_iovec);
    struct iovec* iovs = (struct iovec*)malloc(sizeof(struct iovec) * num_iovec);
    ssize_t arena = mmap(0x1deadb000, 0x200000000, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (arena == -1) {
        printf("Failed to mmap...exiting\n");
        return -1;
        //abort();
    } else {
        printf("Got addr %lx\n", arena);
        fflush(stdout);
    }

    int syscall;
    switch((int)fgetc(ptr) % 4) {
        case 0: syscall = __NR_readv;
        case 1: syscall = __NR_writev;
        case 2: syscall = __NR_preadv;
        case 3: syscall = __NR_pwritev;
    }

    for (int i = 0; i < num_iovec; i++) {
        //do {
            char tmp[8] = {0};
            tmp[0] = fgetc(ptr);
            tmp[1] = fgetc(ptr);
            tmp[2] = fgetc(ptr);
            tmp[3] = fgetc(ptr);
            tmp[4] = fgetc(ptr);
            tmp[5] = fgetc(ptr);
            tmp[6] = fgetc(ptr);
            tmp[7] = fgetc(ptr);
            size_t addr_offset = *(size_t*)tmp;
            tmp[0] = fgetc(ptr);
            tmp[1] = fgetc(ptr);
            tmp[2] = fgetc(ptr);
            tmp[3] = fgetc(ptr);
            tmp[4] = fgetc(ptr);
            tmp[5] = fgetc(ptr);
            tmp[6] = fgetc(ptr);
            tmp[7] = fgetc(ptr);
            size_t len = *(size_t*)tmp;
            if (arena + addr_offset < 0x1deadb000 || arena + addr_offset > 0x1deadb000 + 0x200000000) {
                return -1;
            }
            if (arena + addr_offset + len > 0x1deadb000 + 0x200000000) {
                return -1;
            }
            printf("read addr: %lx, len %lx\n", addr_offset, len);
            //if (addr_offset > 0x200000000) {
            //    abort();
            //}
            //if (len + addr_offset > 0x2000000000) {
            //    abort();
            //}
            iovs[i].iov_base = arena + addr_offset;
            iovs[i].iov_len = len;
    }
    fclose(ptr);
    int fd = open(file_path, O_RDWR);
    if (fd == -1) {
        return 0;
    }
    test_syscall(syscall, fd, iovs, num_iovec, resolved_path);

    return 0;
}

