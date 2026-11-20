// Standalone finite AOT launcher for the compact integer CUDA count reader.
#include "lookup.cuh"

#include <cstdio>
#include <cstdint>
#include <fstream>
#include <stdexcept>
#include <string>
#include <vector>

using namespace compact_count_cuda;

constexpr U64 max_artifact = 4'000'000'000;
constexpr U32 max_probes = 65'536;
constexpr std::size_t chunk_bytes = 8u << 20;

static void check(cudaError_t err) {
    if (err != cudaSuccess) throw std::runtime_error(cudaGetErrorString(err));
}

template<class T> class DeviceBuffer {
    T *address_ = nullptr;
public:
    explicit DeviceBuffer(std::size_t count) { check(cudaMalloc(&address_, count * sizeof(T))); }
    ~DeviceBuffer() { if (address_) cudaFree(address_); }
    DeviceBuffer(const DeviceBuffer &) = delete;
    DeviceBuffer &operator=(const DeviceBuffer &) = delete;
    T *get() { return address_; }
};

static U64 length(std::ifstream &input) {
    input.seekg(0, std::ios::end);
    const auto end = input.tellg();
    if (end < 0) throw std::runtime_error("could not determine input size");
    input.seekg(0);
    return U64(end);
}

template<class T> static void read(std::ifstream &input, T *out, std::size_t count) {
    if (!input.read(reinterpret_cast<char*>(out), count * sizeof(T)))
        throw std::runtime_error("truncated input");
}

static U32 host32(const unsigned char *p) {
    return U32(p[0]) | (U32(p[1]) << 8) | (U32(p[2]) << 16) | (U32(p[3]) << 24);
}

static void validate_layout(std::ifstream &artifact, U64 bytes, const Order *orders) {
    unsigned char main_header[44];
    read(artifact, main_header, sizeof(main_header));
    if (std::string(reinterpret_cast<const char *>(main_header), 8) != "UORKNC02" ||
        host32(main_header + 8) != 10) throw std::runtime_error("wrong compact artifact format");
    U64 offset = sizeof(main_header);
    for (U32 k = 0; k < order_count; ++k) {
        unsigned char header[72];
        artifact.seekg(offset);
        read(artifact, header, sizeof(header));
        const Order &d = orders[k];
        const U32 key_width = (12u * k + 7u) >> 3;
        if (d.keys != offset + sizeof(header) || d.contexts != host32(header + 4) ||
            d.followers != host32(header) || (d.contexts == 0) != (d.followers == 0) ||
            d.rows != d.keys + U64(d.contexts) * (key_width + 4))
            throw std::runtime_error("metadata disagrees with serialized compact section");
        for (U32 index = 0; index < 3; ++index) {
            if (d.discounts[index] > (index + 1) * one)
                throw std::runtime_error("invalid compact discount metadata");
        }
        offset = d.rows + U64(d.followers) * (k == 10 ? 6u : 10u);
        if (offset > bytes) throw std::runtime_error("compact section exceeds artifact");
    }
    if (offset != bytes) throw std::runtime_error("compact artifact length mismatch");
    artifact.seekg(0);
}

static void run(const char *artifact_path, const char *metadata_path, const char *probe_path, const char *out) {
    std::ifstream artifact(artifact_path, std::ios::binary), metadata(metadata_path, std::ios::binary),
        probes(probe_path, std::ios::binary);
    const U64 bytes = length(artifact), probe_bytes = length(probes);
    if (bytes > max_artifact || probe_bytes == 0 || probe_bytes % sizeof(Probe) != 0 ||
        probe_bytes / sizeof(Probe) > max_probes || length(metadata) != order_count * sizeof(Order))
        throw std::runtime_error("too large or malformed count, metadata, or probe tape");
    Order orders[order_count];
    read(metadata, orders, order_count);
    validate_layout(artifact, bytes, orders);
    const U32 count = U32(probe_bytes / sizeof(Probe));
    std::vector<Probe> input(count);
    read(probes, input.data(), count);
    std::vector<OrderResult> result(static_cast<std::size_t>(count) * order_count);
    std::vector<U32> status(count);
    std::vector<unsigned char> chunk(chunk_bytes);
    std::size_t free_bytes = 0, total_bytes = 0;
    check(cudaMemGetInfo(&free_bytes, &total_bytes));
    if (free_bytes < bytes + sizeof(orders) + probe_bytes + result.size() * sizeof(OrderResult)
        + status.size() * sizeof(U32) + (64u << 20))
        throw std::runtime_error("insufficient free VRAM for compact state and bounded batch");
    DeviceBuffer<unsigned char> device_artifact(bytes);
    DeviceBuffer<Order> device_orders(order_count);
    DeviceBuffer<Probe> device_probes(count);
    DeviceBuffer<OrderResult> device_results(result.size());
    DeviceBuffer<U32> device_status(count);
    for (U64 offset = 0; offset < bytes;) {
        const U64 size = bytes - offset < chunk.size() ? bytes - offset : chunk.size();
        read(artifact, chunk.data(), size);
        check(cudaMemcpy(device_artifact.get() + offset, chunk.data(), size, cudaMemcpyHostToDevice));
        offset += size;
    }
    check(cudaMemcpy(device_orders.get(), orders, sizeof(orders), cudaMemcpyHostToDevice));
    check(cudaMemcpy(device_probes.get(), input.data(), probe_bytes, cudaMemcpyHostToDevice));
    lookup<<<(count + 127u) >> 7, 128>>>(
        device_artifact.get(), device_orders.get(), device_probes.get(),
        count, device_results.get(), device_status.get());
    check(cudaGetLastError());
    check(cudaMemcpy(status.data(), device_status.get(), status.size() * sizeof(U32), cudaMemcpyDeviceToHost));
    for (const U32 word : status) {
        if (word != 0) throw std::runtime_error("CUDA reader rejected a compact span or probe");
    }
    check(cudaMemcpy(result.data(), device_results.get(), result.size() * sizeof(OrderResult), cudaMemcpyDeviceToHost));
    std::FILE *file = std::fopen(out, "wbx");
    if (!file) throw std::runtime_error("could not exclusively create result");
    const bool written = std::fwrite(result.data(), sizeof(OrderResult), result.size(), file) == result.size();
    const bool closed = std::fclose(file) == 0;
    if (!written || !closed) {
        std::remove(out);
        throw std::runtime_error("could not seal result words");
    }
}

int main(int argc, char **argv) {
    if (argc != 5) {
        std::fprintf(stderr, "usage: d36-count-device ARTIFACT ORDERS PROBES NEW-RESULT\n");
        return 2;
    }
    const std::uint16_t byte_order = 1;
    if (*reinterpret_cast<const unsigned char *>(&byte_order) != 1) return 2;
    try {
        run(argv[1], argv[2], argv[3], argv[4]);
        return 0;
    } catch (const std::exception &error) {
        std::fprintf(stderr, "%s\n", error.what());
        return 1;
    }
}
