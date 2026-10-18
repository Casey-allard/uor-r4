/* UOR wire interposer: transparently compress configured TCP connections by speaking the
 * adapter/wire-socket protocol to a gateway. DYLD_INSERT_LIBRARIES (macOS) or LD_PRELOAD (Linux).
 *
 * Config: WIRE_ROUTES="desthost:port=gatewayhost:port[#route];..." (repeatable entries).
 * Only IPv4 destinations are matched; unconfigured sockets pass through untouched.
 * Blocking sockets only. Uses the exact wire framing of tools/wire_adapter.py / wire_socket.py:
 *   MAGIC, u8 route length, route, MAGIC reply, then u32 frames (bit31 = raw), zero = EOS.
 */
#define _GNU_SOURCE
#include <arpa/inet.h>
#include <dlfcn.h>
#include <errno.h>
#include <poll.h>
#include <stdatomic.h>
#include <netinet/in.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/syscall.h>
#include <unistd.h>
#include <zlib.h>

#define MAGIC "UORADP01"
#define MAGIC_LEN 8
#define HEADER 4
#define RAW_FLAG 0x80000000u
#define MAX_FD 1024
#define MAX_ROUTES 16
#define MAX_ROUTE 255
#define LEVEL 6
#define CHUNK 65536

#if defined(__APPLE__)
#define WIRE_API(name) wire_##name
#else
#define WIRE_API(name) name
#endif

typedef struct {
    struct sockaddr_in dest;
    struct sockaddr_in gateway;
    char route[MAX_ROUTE + 1];
} Route;

static Route routes[MAX_ROUTES];
static int route_count = 0;
static int routes_parsed = 0;
static int debug_on = 0;

#if defined(__APPLE__)
/* Raw syscalls bypass our own interposed libc names (no dlsym re-entry, no stdio recursion). */
static int sys_connect(int fd, const struct sockaddr *addr, socklen_t len) {
    return (int)syscall(SYS_connect, fd, addr, len);
}
static ssize_t sys_send(int fd, const void *buf, size_t len, int flags) {
    (void)flags;
    return (ssize_t)syscall(SYS_write, fd, buf, len);
}
static ssize_t sys_recv(int fd, void *buf, size_t len, int flags) {
    (void)flags;
    return (ssize_t)syscall(SYS_read, fd, buf, len);
}
static int sys_shutdown(int fd, int how) {
    return (int)syscall(SYS_shutdown, fd, how);
}
static int sys_close(int fd) {
    return (int)syscall(SYS_close, fd);
}
static ssize_t sys_read(int fd, void *buf, size_t len) {
    return (ssize_t)syscall(SYS_read, fd, buf, len);
}
static ssize_t sys_write(int fd, const void *buf, size_t len) {
    return (ssize_t)syscall(SYS_write, fd, buf, len);
}
#else
static int (*real_connect)(int, const struct sockaddr *, socklen_t);
static ssize_t (*real_send)(int, const void *, size_t, int);
static ssize_t (*real_recv)(int, void *, size_t, int);
static ssize_t (*real_read)(int, void *, size_t);
static ssize_t (*real_write)(int, const void *, size_t);
static int (*real_close)(int);
static int (*real_shutdown)(int, int);

static int sys_connect(int fd, const struct sockaddr *addr, socklen_t len) {
    return real_connect(fd, addr, len);
}
static ssize_t sys_send(int fd, const void *buf, size_t len, int flags) {
    return real_send(fd, buf, len, flags);
}
static ssize_t sys_recv(int fd, void *buf, size_t len, int flags) {
    return real_recv(fd, buf, len, flags);
}
static int sys_shutdown(int fd, int how) {
    return real_shutdown(fd, how);
}
static int sys_close(int fd) {
    return real_close(fd);
}
static ssize_t sys_read(int fd, void *buf, size_t len) {
    return (ssize_t)syscall(SYS_read, fd, buf, len);
}
static ssize_t sys_write(int fd, const void *buf, size_t len) {
    return (ssize_t)syscall(SYS_write, fd, buf, len);
}
#endif

typedef struct {
    _Atomic int active;
    pthread_mutex_t lock;
    pthread_mutex_t io_lock;   /* serializes recv only; never held by close/shutdown */
    int send_mode;   /* -1 undecided, 0 compressed, 1 raw */
    z_stream def;
    int def_init;
    int recv_mode;
    z_stream inf;
    int inf_init;
    unsigned char *rbuf;
    size_t rbuf_len;
    size_t rbuf_off;
    _Atomic int eof;
    unsigned generation;
    int eof_sent;
} FdState;

static FdState fds[MAX_FD];
static pthread_mutex_t stats_lock = PTHREAD_MUTEX_INITIALIZER;
static unsigned long long plain_in, plain_out, wire_in, wire_out, connections;


static int parse_addr(const char *text, struct sockaddr_in *out) {
    char host[256];
    const char *colon = strrchr(text, ':');
    if (!colon || colon == text) return -1;
    size_t len = (size_t)(colon - text);
    if (len >= sizeof(host)) return -1;
    memcpy(host, text, len);
    host[len] = 0;
    int port = atoi(colon + 1);
    if (port <= 0 || port > 65535) return -1;
    memset(out, 0, sizeof(*out));
    out->sin_family = AF_INET;
    out->sin_port = htons((uint16_t)port);
    /* Numeric IPv4 only: resolving hostnames here could re-enter the intercepted read/connect. */
    if (inet_pton(AF_INET, host, &out->sin_addr) == 1) return 0;
    return -1;
}

static void parse_routes(void) {
    debug_on = getenv("WIRE_DEBUG") != NULL;
    const char *spec = getenv("WIRE_ROUTES");
    if (!spec) {
        const char *match = getenv("WIRE_MATCH");
        const char *gateway = getenv("WIRE_GATEWAY");
        const char *route = getenv("WIRE_ROUTE");
        static char fallback[1024];
        if (match && gateway) {
            snprintf(fallback, sizeof(fallback), "%s=%s%s%s", match, gateway,
                     (route && *route) ? "#" : "", (route && *route) ? route : "");
            spec = fallback;
        }
    }
    if (!spec) return;
    char *copy = strdup(spec);
    if (!copy) return;
    char *save = NULL;
    char *entry = strtok_r(copy, ";", &save);
    while (entry && route_count < MAX_ROUTES) {
        char *equals = strchr(entry, '=');
        if (equals) {
            *equals = 0;
            char *target = equals + 1;
            char *hash = strchr(target, '#');
            if (hash) {
                *hash = 0;
                snprintf(routes[route_count].route, MAX_ROUTE + 1, "%s", hash + 1);
            } else {
                routes[route_count].route[0] = 0;
            }
            if (parse_addr(entry, &routes[route_count].dest) == 0 &&
                parse_addr(target, &routes[route_count].gateway) == 0) {
                route_count += 1;
            } else if (debug_on) {
                fprintf(stderr, "wireinterposer: bad route entry %s\n", entry);
            }
        }
        entry = strtok_r(NULL, ";", &save);
    }
    free(copy);
    routes_parsed = 1;
}

static void init_symbols(void) {
#if !defined(__APPLE__)
    real_connect = dlsym(RTLD_NEXT, "connect");
    real_send = dlsym(RTLD_NEXT, "send");
    real_recv = dlsym(RTLD_NEXT, "recv");
    real_read = dlsym(RTLD_NEXT, "read");
    real_write = dlsym(RTLD_NEXT, "write");
    real_close = dlsym(RTLD_NEXT, "close");
    real_shutdown = dlsym(RTLD_NEXT, "shutdown");
#endif
}

static void dump_stats(void) {
    pthread_mutex_lock(&stats_lock);
    double plain = (double)(plain_in + plain_out);
    double wire = (double)(wire_in + wire_out);
    double ratio = plain > 0.0 ? wire / plain : 0.0;
    char line[512];
    snprintf(line, sizeof(line),
             "WIRE_INTERPOSER_STATS {\"connections\":%llu,\"plain_in\":%llu,\"plain_out\":%llu,"
             "\"wire_in\":%llu,\"wire_out\":%llu,\"ratio\":%.4f}",
             connections, plain_in, plain_out, wire_in, wire_out, ratio);
    fprintf(stderr, "%s\n", line);
    const char *path = getenv("WIRE_STATS_FILE");
    if (path) {
        FILE *file = fopen(path, "w");
        if (file) {
            fprintf(file, "%s\n", line);
            fclose(file);
        }
    }
    pthread_mutex_unlock(&stats_lock);
}

static volatile int g_ready = 0;

static volatile int child_after_fork = 0;

static void child_after_fork_handler(void) {
    child_after_fork = 1;
}

static void parent_after_fork_handler(void) {
    child_after_fork = 0;
}

static void reinit_locks_after_fork(void) {
    int i;
    for (i = 0; i < MAX_FD; i += 1) {
        pthread_mutex_init(&fds[i].lock, NULL);
        pthread_mutex_init(&fds[i].io_lock, NULL);
    }
    pthread_mutex_init(&stats_lock, NULL);
}

static void wire_interposer_ctor(void) __attribute__((constructor));
static void wire_interposer_ctor(void) {
    init_symbols();
    parse_routes();
    int i;
    for (i = 0; i < MAX_FD; i += 1) {
        pthread_mutex_init(&fds[i].lock, NULL);
        pthread_mutex_init(&fds[i].io_lock, NULL);
    }
    pthread_atfork(NULL, parent_after_fork_handler, reinit_locks_after_fork);
    g_ready = 1;
    atexit(dump_stats);
}

static Route *match_route(const struct sockaddr *addr, socklen_t len) {
    if (!addr || len < (socklen_t)sizeof(struct sockaddr_in) || addr->sa_family != AF_INET) return NULL;
    const struct sockaddr_in *in = (const struct sockaddr_in *)addr;
    int i;
    for (i = 0; i < route_count; i += 1) {
        if (routes[i].dest.sin_port == in->sin_port &&
            routes[i].dest.sin_addr.s_addr == in->sin_addr.s_addr) return &routes[i];
    }
    return NULL;
}



static ssize_t send_all(int fd, const void *buffer, size_t size) {
    const unsigned char *cursor = buffer;
    size_t left = size;
    while (left > 0) {
        ssize_t sent = sys_send(fd, cursor, left, 0);
        if (sent < 0) {
            if (errno == EINTR) continue;
            return -1;
        }
        cursor += sent;
        left -= (size_t)sent;
    }
    return (ssize_t)size;
}

static ssize_t recv_into(int fd, void *buffer, size_t size) {
    unsigned char *cursor = buffer;
    size_t left = size;
    while (left > 0) {
        ssize_t got = sys_recv(fd, cursor, left, 0);
        if (got < 0) {
            if (errno == EINTR) continue;
            return -1;
        }
        if (got == 0) return (ssize_t)(size - left);
        cursor += got;
        left -= (size_t)got;
    }
    return (ssize_t)size;
}

static ssize_t sys_recv_nowait(int fd, void *buf, size_t len) {
    return (ssize_t)syscall(SYS_recvfrom, fd, buf, len, MSG_DONTWAIT, NULL, NULL);
}

static int wait_readable(FdState *state, int fd) {
    /* Readiness wait only; reads are non-blocking chunks in read_exact_active so
       close() can stop a reader without shutdown() (fork-shared sockets). */
    struct pollfd pfd;
    pfd.fd = fd;
    pfd.events = POLLIN;
    for (;;) {
        if (!atomic_load(&state->active)) return -1;
        pfd.revents = 0;
        int rc = poll(&pfd, 1, 50);
        if (rc > 0) {
            if (!atomic_load(&state->active)) return -1;
            if (pfd.revents & POLLNVAL) return -1;
            if (pfd.revents & (POLLIN | POLLHUP | POLLERR)) return 0;
            continue;
        }
        if (rc == 0 || errno == EINTR) continue;
        return -1;
    }
}

static ssize_t read_exact_active(FdState *state, int fd, void *buffer, size_t size) {
    unsigned char *cursor = buffer;
    size_t left = size;
    while (left > 0) {
        if (!atomic_load(&state->active)) return -1;
        ssize_t got = sys_recv_nowait(fd, cursor, left);
        if (got > 0) {
            cursor += got;
            left -= (size_t)got;
            continue;
        }
        if (got == 0) return (ssize_t)(size - left);
        if (errno == EINTR) continue;
        if (errno == EAGAIN || errno == EWOULDBLOCK) {
            if (wait_readable(state, fd) < 0) return -1;
            continue;
        }
        return -1;
    }
    return (ssize_t)size;
}

static int handshake(int fd, const char *route) {
    unsigned char frame[MAGIC_LEN + 1 + MAX_ROUTE];
    size_t length = (size_t)strlen(route);
    if (length > MAX_ROUTE) length = MAX_ROUTE;
    memcpy(frame, MAGIC, MAGIC_LEN);
    frame[MAGIC_LEN] = (unsigned char)length;
    memcpy(frame + MAGIC_LEN + 1, route, length);
    if (send_all(fd, frame, MAGIC_LEN + 1 + length) < 0) return -1;
    unsigned char reply[MAGIC_LEN];
    if (recv_into(fd, reply, MAGIC_LEN) != MAGIC_LEN) return -1;
    if (memcmp(reply, MAGIC, MAGIC_LEN) != 0) return -1;
    return 0;
}

static int ensure_frame_buffer(FdState *state, size_t needed) {
    if (state->rbuf_len >= needed) return 0;
    unsigned char *grown = state->rbuf ? realloc(state->rbuf, needed) : malloc(needed);
    if (!grown) return -1;
    state->rbuf = grown;
    state->rbuf_len = needed;
    return 0;
}

static int write_frame(FdState *state, int fd, const void *data, size_t size) {
    unsigned char header[HEADER];
    unsigned char *compressed = NULL;
    const void *payload = data;
    size_t payload_size = size;
    if (state->send_mode < 0) {
        size_t bound = compressBound((uLong)size);
        compressed = malloc(bound);
        if (!compressed) return -1;
        uLongf dest_len = (uLongf)bound;
        int rc = compress2(compressed, &dest_len, (const Bytef *)data, (uLong)size, LEVEL);
        int incompressible = (rc == Z_OK && (size >= 4096) && ((size_t)dest_len >= size));
        free(compressed);
        compressed = NULL;
        if (incompressible) {
            state->send_mode = 1;
        } else {
            state->send_mode = 0;
            memset(&state->def, 0, sizeof(state->def));
            if (deflateInit(&state->def, LEVEL) != Z_OK) return -1;
            state->def_init = 1;
        }
    }
    if (state->send_mode == 1) {
        uint32_t word = RAW_FLAG | (uint32_t)size;
        header[0] = (unsigned char)(word >> 24);
        header[1] = (unsigned char)(word >> 16);
        header[2] = (unsigned char)(word >> 8);
        header[3] = (unsigned char)word;
        if (send_all(fd, header, HEADER) < 0) return -1;
        if (send_all(fd, data, size) < 0) return -1;
        pthread_mutex_lock(&stats_lock);
        plain_in += size;
        wire_out += HEADER + size;
        pthread_mutex_unlock(&stats_lock);
        return 0;
    }
    size_t bound = (size_t)compressBound((uLong)size) + 64;
    unsigned char *out = malloc(bound);
    if (!out) return -1;
    state->def.next_in = (Bytef *)data;
    state->def.avail_in = (uInt)size;
    state->def.next_out = out;
    state->def.avail_out = (uInt)bound;
    int rc = deflate(&state->def, Z_SYNC_FLUSH);
    if (rc != Z_OK && rc != Z_STREAM_END) {
        free(out);
        return -1;
    }
    size_t produced = bound - state->def.avail_out;
    uint32_t word = (uint32_t)produced;
    header[0] = (unsigned char)(word >> 24);
    header[1] = (unsigned char)(word >> 16);
    header[2] = (unsigned char)(word >> 8);
    header[3] = (unsigned char)word;
    if (send_all(fd, header, HEADER) < 0 || send_all(fd, out, produced) < 0) {
        free(out);
        return -1;
    }
    free(out);
    pthread_mutex_lock(&stats_lock);
    plain_in += size;
    wire_out += HEADER + produced;
    pthread_mutex_unlock(&stats_lock);
    return 0;
}

static ssize_t read_frame(FdState *state, int fd, void *buffer, size_t size) {
    while (state->rbuf_off >= state->rbuf_len) {
        if (state->eof) return 0;
        unsigned char header[HEADER];
        ssize_t got = read_exact_active(state, fd, header, HEADER);
        if (got != (ssize_t)HEADER) {
            state->eof = 1;
            return 0;
        }
        uint32_t word = ((uint32_t)header[0] << 24) | ((uint32_t)header[1] << 16) |
                        ((uint32_t)header[2] << 8) | (uint32_t)header[3];
        int raw = (word & RAW_FLAG) != 0;
        size_t frame_size = (size_t)(word & ~RAW_FLAG);
        pthread_mutex_lock(&stats_lock);
        wire_in += HEADER + frame_size;
        pthread_mutex_unlock(&stats_lock);
        if (frame_size == 0) {
            state->eof = 1;
            return 0;
        }
        unsigned char *blob = malloc(frame_size);
        if (!blob) return -1;
        got = read_exact_active(state, fd, blob, frame_size);
        if (got != (ssize_t)frame_size) {
            free(blob);
            state->eof = 1;
            return 0;
        }
        if (state->recv_mode < 0) state->recv_mode = raw ? 1 : 0;
        state->rbuf_off = 0;
        state->rbuf_len = 0;
        if (state->recv_mode == 1) {
            if (ensure_frame_buffer(state, frame_size) < 0) {
                free(blob);
                return -1;
            }
            memcpy(state->rbuf, blob, frame_size);
            state->rbuf_len = frame_size;
            pthread_mutex_lock(&stats_lock);
            plain_out += frame_size;
            pthread_mutex_unlock(&stats_lock);
        } else {
            if (!state->inf_init) {
                memset(&state->inf, 0, sizeof(state->inf));
                if (inflateInit(&state->inf) != Z_OK) {
                    free(blob);
                    return -1;
                }
                state->inf_init = 1;
            }
            size_t out_size = frame_size * 64 + CHUNK;
            if (out_size < CHUNK) out_size = CHUNK;
            unsigned char *produced = malloc(out_size);
            if (!produced) {
                free(blob);
                return -1;
            }
            size_t used = 0;
            state->inf.next_in = blob;
            state->inf.avail_in = (uInt)frame_size;
            while (state->inf.avail_in > 0) {
                if (used + CHUNK > out_size) {
                    out_size *= 2;
                    unsigned char *grown = realloc(produced, out_size);
                    if (!grown) {
                        free(produced);
                        free(blob);
                        return -1;
                    }
                    produced = grown;
                }
                state->inf.next_out = produced + used;
                state->inf.avail_out = CHUNK;
                int rc = inflate(&state->inf, Z_SYNC_FLUSH);
                if (rc != Z_OK && rc != Z_STREAM_END && rc != Z_BUF_ERROR) {
                    free(produced);
                    free(blob);
                    return -1;
                }
                used += CHUNK - state->inf.avail_out;
                if (rc == Z_STREAM_END) break;
                if (rc == Z_BUF_ERROR && state->inf.avail_in == 0) break;
            }
            if (ensure_frame_buffer(state, used) < 0) {
                free(produced);
                free(blob);
                return -1;
            }
            memcpy(state->rbuf, produced, used);
            state->rbuf_len = used;
            state->rbuf_off = 0;
            free(produced);
            pthread_mutex_lock(&stats_lock);
            plain_out += used;
            pthread_mutex_unlock(&stats_lock);
        }
        free(blob);
    }
    size_t available = state->rbuf_len - state->rbuf_off;
    size_t take = available < size ? available : size;
    memcpy(buffer, state->rbuf + state->rbuf_off, take);
    state->rbuf_off += take;
    return (ssize_t)take;
}

static void reset_state(FdState *state) {
    if (child_after_fork) {
        state->rbuf = NULL;
        state->rbuf_len = 0;
        state->rbuf_off = 0;
        state->def_init = 0;
        state->inf_init = 0;
        state->active = 0;
        state->send_mode = -1;
        state->recv_mode = -1;
        state->eof = 0;
        state->eof_sent = 0;
        return;
    }
    if (state->def_init) {
        deflateEnd(&state->def);
        state->def_init = 0;
    }
    if (state->inf_init) {
        inflateEnd(&state->inf);
        state->inf_init = 0;
    }
    free(state->rbuf);
    state->rbuf = NULL;
    state->rbuf_len = 0;
    state->rbuf_off = 0;
    state->active = 0;
    state->send_mode = -1;
    state->recv_mode = -1;
    state->eof = 0;
    state->eof_sent = 0;
}

int WIRE_API(connect)(int fd, const struct sockaddr *addr, socklen_t len) {
    if (!g_ready) return sys_connect(fd, addr, len);
    
    Route *route = match_route(addr, len);
    if (!route) return sys_connect(fd, addr, len);
    if (fd < 0 || fd >= MAX_FD) {
        errno = EMFILE;
        return -1;
    }
    FdState *state = &fds[fd];
    pthread_mutex_lock(&state->lock);
    int need_reset = !state->active;
    pthread_mutex_unlock(&state->lock);
    if (need_reset) {
        /* Wait for a possibly still-running recv on this fd to leave io_lock before
           resetting state, so fd reuse cannot race a blocked reader. */
        pthread_mutex_lock(&state->io_lock);
        pthread_mutex_lock(&state->lock);
        if (!state->active) reset_state(state);
        pthread_mutex_unlock(&state->lock);
        pthread_mutex_unlock(&state->io_lock);
    }
    pthread_mutex_lock(&state->lock);
    if (sys_connect(fd, (const struct sockaddr *)&route->gateway, sizeof(route->gateway)) < 0) {
        pthread_mutex_unlock(&state->lock);
        return -1;
    }
    if (handshake(fd, route->route) < 0) {
        sys_close(fd);
        errno = ECONNREFUSED;
        pthread_mutex_unlock(&state->lock);
        return -1;
    }
    state->active = 1;
    state->generation += 1;
    state->send_mode = -1;
    state->recv_mode = -1;
    state->eof = 0;
    state->eof_sent = 0;
    state->rbuf = NULL;
    state->rbuf_len = 0;
    state->rbuf_off = 0;
    pthread_mutex_unlock(&state->lock);
    pthread_mutex_lock(&stats_lock);
    connections += 1;
    pthread_mutex_unlock(&stats_lock);
    return 0;
}

ssize_t WIRE_API(send)(int fd, const void *buffer, size_t size, int flags) {
    if (!g_ready) return sys_send(fd, buffer, size, flags);
    if (fd < 0 || fd >= MAX_FD || flags != 0) return sys_send(fd, buffer, size, flags);
    FdState *state = &fds[fd];
    pthread_mutex_lock(&state->lock);
    int was_active = atomic_load(&state->active);
    unsigned generation = state->generation;
    pthread_mutex_unlock(&state->lock);
    if (!was_active) return sys_send(fd, buffer, size, flags);
    pthread_mutex_lock(&state->lock);
    if (!atomic_load(&state->active) || state->generation != generation || state->eof_sent) {
        pthread_mutex_unlock(&state->lock);
        errno = EPIPE;
        return -1;
    }
    int rc = write_frame(state, fd, buffer, size);
    pthread_mutex_unlock(&state->lock);
    if (rc != 0) return -1;
    return (ssize_t)size;
}

ssize_t WIRE_API(recv)(int fd, void *buffer, size_t size, int flags) {
    if (!g_ready) return sys_recv(fd, buffer, size, flags);
    if (fd < 0 || fd >= MAX_FD || flags != 0 || !atomic_load(&fds[fd].active)) {
        return sys_recv(fd, buffer, size, flags);
    }
    FdState *state = &fds[fd];
    pthread_mutex_lock(&state->lock);
    int was_active = atomic_load(&state->active);
    unsigned generation = state->generation;
    pthread_mutex_unlock(&state->lock);
    if (!was_active) return sys_recv(fd, buffer, size, flags);
    pthread_mutex_lock(&state->io_lock);
    pthread_mutex_lock(&state->lock);
    int valid = atomic_load(&state->active) && state->generation == generation;
    pthread_mutex_unlock(&state->lock);
    if (!valid) {
        pthread_mutex_unlock(&state->io_lock);
        errno = ECONNRESET;
        return -1;
    }
    /* read_frame reads in non-blocking chunks and re-checks active between them, so
       close() wakes it without shutdown() and reused-fd connect waits are bounded. */
    ssize_t result = read_frame(state, fd, buffer, size);
    pthread_mutex_unlock(&state->io_lock);
    return result;
}

ssize_t WIRE_API(read)(int fd, void *buffer, size_t size) {
    if (!g_ready) return sys_read(fd, buffer, size);
    if (fd < 0 || fd >= MAX_FD || !fds[fd].active) return sys_read(fd, buffer, size);
    return WIRE_API(recv)(fd, buffer, size, 0);
}

ssize_t WIRE_API(write)(int fd, const void *buffer, size_t size) {
    if (!g_ready) return sys_write(fd, buffer, size);
    if (fd < 0 || fd >= MAX_FD || !fds[fd].active) return sys_write(fd, buffer, size);
    return WIRE_API(send)(fd, buffer, size, 0);
}

int WIRE_API(shutdown)(int fd, int how) {
    if (!g_ready) return sys_shutdown(fd, how);
    if (fd < 0 || fd >= MAX_FD) return sys_shutdown(fd, how);
    FdState *state = &fds[fd];
    pthread_mutex_lock(&state->lock);
    int was_active = atomic_load(&state->active);
    unsigned generation = state->generation;
    pthread_mutex_unlock(&state->lock);
    if (!was_active) return sys_shutdown(fd, how);
    pthread_mutex_lock(&state->lock);
    if (!atomic_load(&state->active) || state->generation != generation) {
        pthread_mutex_unlock(&state->lock);
        errno = EPIPE;
        return -1;
    }
    int rc = 0;
    if ((how == SHUT_WR || how == SHUT_RDWR) && !state->eof_sent) {
        unsigned char zero[HEADER] = {0, 0, 0, 0};
        if (send_all(fd, zero, HEADER) < 0) rc = -1;
        state->eof_sent = 1;
    }
    if (how == SHUT_RD || how == SHUT_RDWR) state->eof = 1;
    if (rc == 0) rc = sys_shutdown(fd, how);
    pthread_mutex_unlock(&state->lock);
    return rc;
}

int WIRE_API(close)(int fd) {
    if (!g_ready) return sys_close(fd);
    if (fd >= 0 && fd < MAX_FD && atomic_load(&fds[fd].active)) {
        FdState *state = &fds[fd];
        pthread_mutex_lock(&state->lock);
        atomic_store(&state->active, 0);
        atomic_store(&state->eof, 1);
        pthread_mutex_unlock(&state->lock);
        /* No shutdown(): it would disturb a socket shared after fork. A reader in
           read_exact_active sees active == 0 between non-blocking chunks and exits.
           A reused-fd connect may additionally wait for the current frame's
           decompression to finish (bounded CPU), but not for further socket I/O. */
#if defined(WIRE_UNSAFE_CLOSE_SHUTDOWN)
        (void)sys_shutdown(fd, SHUT_RDWR);
#endif
    }
    return sys_close(fd);
}

#if defined(__APPLE__)
#define DEFINE_INTERPOSE(replacement, replacee) \
    __attribute__((used)) static struct { const void *replacement; const void *replacee; } \
    _interpose_##replacee __attribute__((section("__DATA,__interpose"))) = { \
        (const void *)(uintptr_t)(replacement), (const void *)(uintptr_t)(replacee) };
DEFINE_INTERPOSE(wire_connect, connect)
DEFINE_INTERPOSE(wire_send, send)
DEFINE_INTERPOSE(wire_recv, recv)
DEFINE_INTERPOSE(wire_shutdown, shutdown)
DEFINE_INTERPOSE(wire_close, close)
#endif
