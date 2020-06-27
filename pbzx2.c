###### WARNING: PRIOR WORK: https://gist.githubusercontent.com/xerub/adf396f479d401b9c0e9/raw/18db6c9211a57f969a3c6063554a3ff82c44e1fa/pbzx2.c

#include <stdio.h>
#include <string.h>
#include <fcntl.h>
#include <unistd.h>
#include <stdlib.h>
#include <lzma.h>

int main(int argc,
    const char * argv[]) {

    // Dumps a pbzx to stdout. Can work as a filter if no argument is specified

    char buffer[1024];
    int fd = 0;

    if (argc < 2) {
        fd = 0;
    } else {
        fd = open(argv[1], O_RDONLY);
        if (fd < 0) {
            perror(argv[1]);
            exit(5);
        }
    }

    read(fd, buffer, 4);
    if (strncmp(buffer, "pbzx", 4)) {
        fprintf(stderr, "Can't find pbzx magic\n");
        exit(0);
    }

    // Now, if it IS a pbzx

    uint64_t length = 0, flags = 0;

    read(fd, & flags, sizeof(uint64_t));
    flags = __builtin_bswap64(flags);

    fprintf(stderr, "Flags: 0x%llx\n", flags);

    int i = 0;
    int off = 0;

    while (flags & 0x01000000) { // have more chunks
        int cat = 0;
        i++;
        read(fd, & flags, sizeof(uint64_t));
        flags = __builtin_bswap64(flags);
        read(fd, & length, sizeof(uint64_t));
        length = __builtin_bswap64(length);

        fprintf(stderr, "Chunk #%d (flags: %llx, length: %lld bytes)\n", i, flags, length);

        // Let's ignore the fact I'm allocating based on user input, etc..
        char * buf = malloc(length);
        read(fd, buf, length);

        // We want the XZ header/footer if it's the payload, but prepare_payload doesn't have that, 
        // so just warn.

        if (strncmp(buf, "\xfd"
                "7zXZ\0", 6)) {
            cat = 1;
            fprintf(stderr, "Warning: Can't find XZ header. This is likely not XZ data.\n");
        } else // if we have the header, we had better have a footer, too
            if (strncmp(buf + length - 2, "YZ", 2)) {
                fprintf(stderr, "Warning: Can't find XZ footer. This is bad.\n");
                exit(1);
            }

        // [xerub] cat/unxz here
        if (cat) {
            write(1, buf, length);
        } else {
            uint64_t memlimit = -1;
            size_t max = ((length > 16 * 1024 * 1024) ? length : (16 * 1024 * 1024)) * 10;
            size_t in_pos = 0;
            size_t out_pos = 0;
            unsigned char * output = malloc(max);
            if (!output) {
                fprintf(stderr, "out of memory. abort\n");
                exit(1);
            }
            int rv = lzma_stream_buffer_decode( & memlimit, 0, NULL, (void * ) buf, & in_pos, length, output, & out_pos, max);
            if (rv) {
                fprintf(stderr, "xz returned %d. abort\n", rv);
                exit(1);
            }
            write(1, output, out_pos);
            free(output);
        }

    }

    return 0;
}
