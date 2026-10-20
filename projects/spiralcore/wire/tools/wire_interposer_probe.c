#include <arpa/inet.h>
#include <netinet/in.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <time.h>
#include <unistd.h>

static int connect_to(const char *text) {
    char host[256];
    const char *colon = strrchr(text, ':');
    if (!colon) return -1;
    size_t len = (size_t)(colon - text);
    memcpy(host, text, len);
    host[len] = 0;
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_port = htons((uint16_t)atoi(colon + 1));
    inet_pton(AF_INET, host, &addr.sin_addr);
    if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
        perror("connect");
        close(fd);
        return -1;
    }
    return fd;
}

int main(int argc, char **argv) {
    if (argc < 3) {
        fprintf(stderr, "usage: probe HOST:PORT text|random BYTES\n");
        return 2;
    }
    size_t size = (size_t)atol(argv[3]);
    unsigned char *payload = malloc(size);
    if (strcmp(argv[2], "random") == 0) {
        unsigned int seed = 20261017u;
        size_t i;
        for (i = 0; i < size; i += 1) payload[i] = (unsigned char)(rand_r(&seed) & 0xff);
    } else {
        const char *line = "{\"kind\":\"probe\",\"payload\":\"abcdef0123456789\"}\n";
        size_t line_len = strlen(line);
        size_t i;
        for (i = 0; i < size; i += 1) payload[i] = (unsigned char)line[i % line_len];
    }
    int fd = connect_to(argv[1]);
    if (fd < 0) return 1;
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);
    size_t sent = 0;
    while (sent < size) {
        ssize_t wrote = send(fd, payload + sent, size - sent, 0);
        if (wrote <= 0) { perror("send"); return 1; }
        sent += (size_t)wrote;
    }
    shutdown(fd, SHUT_WR);
    unsigned char *buffer = malloc(size);
    size_t received = 0;
    while (received < size) {
        ssize_t got = recv(fd, buffer + received, size - received, 0);
        if (got <= 0) break;
        received += (size_t)got;
    }
    clock_gettime(CLOCK_MONOTONIC, &end);
    close(fd);
    double seconds = (double)(end.tv_sec - start.tv_sec) + (double)(end.tv_nsec - start.tv_nsec) / 1e9;
    int equal = received == size && memcmp(buffer, payload, size) == 0;
    printf("PROBE_%s bytes=%lu received=%lu seconds=%.4f mibs=%.1f\n",
           equal ? "OK" : "FAIL", (unsigned long)size, (unsigned long)received, seconds,
           seconds > 0.0 ? (double)size / seconds / (1024.0 * 1024.0) : 0.0);
    return equal ? 0 : 1;
}
