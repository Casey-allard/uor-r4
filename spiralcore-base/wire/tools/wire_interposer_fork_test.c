/* Stock-binary fork-safety check for the wire interposer.
 *
 * Opens a routed connection, verifies one round trip, forks; the child closes its
 * inherited descriptor and exits. The parent must continue on the same connection
 * and complete a second, larger round trip. A shutdown() in close() (the old
 * behaviour) breaks the parent; a fork-safe close must keep it working.
 */
#include <arpa/inet.h>
#include <errno.h>
#include <pthread.h>
#include <signal.h>
#include <stdatomic.h>
#include <time.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/wait.h>
#include <unistd.h>


static atomic_int reader_done = 0;
static atomic_int reader_started = 0;
static atomic_int reader_result = 0;
static atomic_int reader_errno = 0;

static void *blocked_reader(void *arg) {
    int fd = *(int *)arg;
    unsigned char byte;
    atomic_store(&reader_started, 1);
    errno = 0;
    ssize_t result = recv(fd, &byte, 1, 0);
    atomic_store(&reader_result, (int)result);
    atomic_store(&reader_errno, errno);
    atomic_store(&reader_done, 1);
    return NULL;
}

static int parse_addr(const char *text, struct sockaddr_in *out) {
    char host[64];
    const char *colon = strrchr(text, ':');
    if (!colon) return -1;
    size_t length = (size_t)(colon - text);
    if (length >= sizeof(host)) return -1;
    memcpy(host, text, length);
    host[length] = '\0';
    memset(out, 0, sizeof(*out));
    out->sin_family = AF_INET;
    out->sin_port = htons((unsigned short)atoi(colon + 1));
    return inet_pton(AF_INET, host, &out->sin_addr) == 1 ? 0 : -1;
}

static int send_all_fd(int fd, const unsigned char *buffer, size_t size) {
    size_t sent = 0;
    while (sent < size) {
        ssize_t chunk = send(fd, buffer + sent, size - sent, 0);
        if (chunk <= 0) return -1;
        sent += (size_t)chunk;
    }
    return 0;
}

static int recv_all_fd(int fd, unsigned char *buffer, size_t size) {
    size_t got = 0;
    while (got < size) {
        ssize_t chunk = recv(fd, buffer + got, size - got, 0);
        if (chunk <= 0) return -1;
        got += (size_t)chunk;
    }
    return 0;
}

static void fill(unsigned char *buffer, size_t size) {
    size_t index;
    for (index = 0; index < size; index += 1) buffer[index] = (unsigned char)(index * 31u + 7u);
}

int main(int argc, char **argv) {
    signal(SIGPIPE, SIG_IGN);
    alarm(15);
    if (argc < 3) {
        fprintf(stderr, "usage: %s HOST:PORT BYTES\n", argv[0]);
        return 2;
    }
    struct sockaddr_in address;
    if (parse_addr(argv[1], &address) != 0) {
        fprintf(stderr, "bad address\n");
        return 2;
    }
    size_t size = (size_t)strtoul(argv[2], NULL, 10);
    if (size < 1024) size = 1024;
    size_t total = size * 2;
    unsigned char *payload = malloc(total);
    unsigned char *echo = malloc(total);
    if (!payload || !echo) return 3;
    fill(payload, total);
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) return 3;
    if (connect(fd, (struct sockaddr *)&address, sizeof(address)) != 0) {
        perror("connect");
        return 4;
    }
    if (send_all_fd(fd, payload, size) != 0 || recv_all_fd(fd, echo, size) != 0 ||
        memcmp(payload, echo, size) != 0) {
        fprintf(stderr, "first round trip failed\n");
        return 5;
    }
    pid_t child = fork();
    if (child == 0) {
        close(fd); /* must not disturb the parent's shared connection */
        _exit(0);
    }
    if (child < 0) return 6;
    int status = 0;
    waitpid(child, &status, 0);
    if (send_all_fd(fd, payload, total) != 0 || recv_all_fd(fd, echo, total) != 0 ||
        memcmp(payload, echo, total) != 0) {
        fprintf(stderr, "post-fork round trip failed\n");
        return 7;
    }
    signal(SIGPIPE, SIG_IGN);
    alarm(15);
    /* Phase 2: close must wake a reader blocked on a routed fd, and the reused fd must
       accept a new routed connection immediately. */
    int fd2 = socket(AF_INET, SOCK_STREAM, 0);
    if (fd2 < 0) return 8;
    if (connect(fd2, (struct sockaddr *)&address, sizeof(address)) != 0) return 8;
    int thread_fd = fd2;
    pthread_t thread;
    atomic_store(&reader_done, 0);
    if (pthread_create(&thread, NULL, blocked_reader, &thread_fd) != 0) return 8;
    usleep(100000);
    close(fd2);
    int waited = 0;
    while (!atomic_load(&reader_done) && waited < 3000) {
        usleep(10000);
        waited += 10;
    }
    if (!atomic_load(&reader_done)) {
        fprintf(stderr, "close did not wake a blocked reader\n");
        return 9;
    }
    pthread_join(thread, NULL);
    int fd3 = socket(AF_INET, SOCK_STREAM, 0);
    if (fd3 < 0) return 10;
    if (connect(fd3, (struct sockaddr *)&address, sizeof(address)) != 0) {
        fprintf(stderr, "reused-fd connect failed\n");
        return 10;
    }
    if (send_all_fd(fd3, payload, size) != 0 || recv_all_fd(fd3, echo, size) != 0 ||
        memcmp(payload, echo, size) != 0) {
        fprintf(stderr, "reused-fd round trip failed\n");
        return 11;
    }
    close(fd3);
    close(fd);
    printf("FORK_TEST_OK bytes=%zu interrupt_ok=true reused_fd_ok=true reader_result=%d\n", total, atomic_load(&reader_result));
    free(payload);
    free(echo);
    return 0;
}
