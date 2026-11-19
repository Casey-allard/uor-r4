#include <cuda_runtime.h>

#include <array>
#include <cstdint>
#include <cstdio>
#include <fstream>
#include <iostream>
#include <limits>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

constexpr int dimensions = 32;
constexpr int classes = 3;
constexpr int max_rows = 1'000'000;
constexpr char magic[] = "D24ROLE1";

struct Candidate {
    int32_t coordinate[dimensions];
};

struct Prototype {
    int64_t count;
    int64_t sum[dimensions];
};

struct Bank {
    Prototype prototype[classes];
};

struct Result {
    int64_t numerator[classes];
    int64_t role;
};

static_assert(sizeof(Bank) == 792);
static_assert(sizeof(Result) == 32);

__host__ __device__ int compare(int64_t left, int64_t left_count,
                               int64_t right, int64_t right_count) {
    // The cross products fit in 77 bits at the declared row and coordinate bounds.
    const auto a = static_cast<unsigned __int128>(left) * right_count;
    const auto b = static_cast<unsigned __int128>(right) * left_count;
    return (a > b) - (a < b);
}

__host__ __device__ Result classify(const Candidate& candidate, const Bank& bank) {
    Result result{};
    int best = 0;
    bool tied = false;
    for (int role = 0; role < classes; ++role) {
        const auto& prototype = bank.prototype[role];
        for (int axis = 0; axis < dimensions; ++axis) {
            int64_t delta = prototype.count * candidate.coordinate[axis] - prototype.sum[axis];
            result.numerator[role] += delta < 0 ? -delta : delta;
        }
        if (role == 0) {
            continue;
        }
        int order = compare(result.numerator[role], prototype.count,
                            result.numerator[best], bank.prototype[best].count);
        if (order < 0) {
            best = role;
            tied = false;
        } else if (order == 0) {
            tied = true;
        }
    }
    result.role = tied ? -1 : best;
    return result;
}

__global__ void fit_kernel(const Candidate* rows, const int32_t* labels, int size, Bank* bank) {
    int role = blockIdx.x;
    int axis = threadIdx.x;
    int64_t count = 0;
    int64_t sum = 0;
    for (int row = 0; row < size; ++row) {
        if (labels[row] == role) {
            ++count;
            sum += rows[row].coordinate[axis];
        }
    }
    if (axis == 0) {
        bank->prototype[role].count = count;
    }
    bank->prototype[role].sum[axis] = sum;
}

__global__ void predict_kernel(const Candidate* rows, int size, const Bank* bank, Result* results) {
    int row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row < size) {
        results[row] = classify(rows[row], *bank);
    }
}

void cuda_check(cudaError_t status, const char* operation) {
    if (status != cudaSuccess) {
        throw std::runtime_error(std::string(operation) + ": " + cudaGetErrorString(status));
    }
}

template <typename T>
class DeviceBuffer {
public:
    explicit DeviceBuffer(size_t count) : bytes(count * sizeof(T)) {
        cuda_check(cudaMalloc(&ptr, bytes), "cudaMalloc");
    }

    DeviceBuffer(const DeviceBuffer&) = delete;
    DeviceBuffer& operator=(const DeviceBuffer&) = delete;

    ~DeviceBuffer() {
        cudaFree(ptr);
    }

    T* data() const {
        return ptr;
    }

    void upload(const T* input) {
        cuda_check(cudaMemcpy(ptr, input, bytes, cudaMemcpyHostToDevice), "upload");
    }

    void download(T* output) const {
        cuda_check(cudaMemcpy(output, ptr, bytes, cudaMemcpyDeviceToHost), "download");
    }

private:
    T* ptr = nullptr;
    size_t bytes;
};

void validate(const Bank& bank) {
    int64_t total = 0;
    for (const auto& prototype : bank.prototype) {
        if (prototype.count < 1 || prototype.count > max_rows) {
            throw std::runtime_error("Each role requires 1 to 1000000 training candidates");
        }
        total += prototype.count;
        for (int64_t sum : prototype.sum) {
            if (sum < prototype.count * std::numeric_limits<int32_t>::min() ||
                sum > prototype.count * std::numeric_limits<int32_t>::max()) {
                throw std::runtime_error("Invalid prototype coordinate sum");
            }
        }
    }
    if (total > max_rows) {
        throw std::runtime_error("Bank exceeds the training candidate limit");
    }
}

Bank fit(const std::vector<Candidate>& rows, const std::vector<int32_t>& labels) {
    DeviceBuffer<Candidate> gpu_rows(rows.size());
    DeviceBuffer<int32_t> gpu_labels(labels.size());
    DeviceBuffer<Bank> gpu_bank(1);
    gpu_rows.upload(rows.data());
    gpu_labels.upload(labels.data());
    fit_kernel<<<classes, dimensions>>>(gpu_rows.data(), gpu_labels.data(), rows.size(), gpu_bank.data());
    cuda_check(cudaGetLastError(), "fit launch");
    Bank bank{};
    gpu_bank.download(&bank);
    validate(bank);
    return bank;
}

std::vector<Result> predict(const std::vector<Candidate>& rows, const Bank& bank) {
    DeviceBuffer<Candidate> gpu_rows(rows.size());
    DeviceBuffer<Bank> gpu_bank(1);
    DeviceBuffer<Result> gpu_results(rows.size());
    gpu_rows.upload(rows.data());
    gpu_bank.upload(&bank);
    constexpr int block = 128;
    predict_kernel<<<(rows.size() + block - 1) / block, block>>>(
        gpu_rows.data(), rows.size(), gpu_bank.data(), gpu_results.data());
    cuda_check(cudaGetLastError(), "predict launch");
    std::vector<Result> results(rows.size());
    gpu_results.download(results.data());
    return results;
}

std::ifstream input(const char* path) {
    std::ifstream stream(path, std::ios::binary);
    if (!stream) {
        throw std::runtime_error(std::string("Cannot open ") + path);
    }
    return stream;
}

void require(bool success, const char* message) {
    if (!success) {
        throw std::runtime_error(message);
    }
}

void write_once(const char* path, const std::string& bytes) {
    std::FILE* file = std::fopen(path, "wbx");
    require(file != nullptr, "Cannot create output. Choose a new output path.");
    size_t written = std::fwrite(bytes.data(), 1, bytes.size(), file);
    int closed = std::fclose(file);
    require(written == bytes.size() && closed == 0, "Incomplete output write");
}

void append_word(std::string& bytes, int64_t value) {
    uint64_t word = static_cast<uint64_t>(value);
    for (int shift = 0; shift < 64; shift += 8) {
        bytes.push_back(static_cast<char>((word >> shift) & 0xff));
    }
}

int64_t read_word(std::istream& stream) {
    uint64_t word = 0;
    for (int shift = 0; shift < 64; shift += 8) {
        int byte = stream.get();
        require(byte != EOF, "Truncated state");
        word |= static_cast<uint64_t>(byte) << shift;
    }
    constexpr auto max = static_cast<uint64_t>(std::numeric_limits<int64_t>::max());
    return word <= max ? static_cast<int64_t>(word) : -1 - static_cast<int64_t>(~word);
}

void save(const char* path, const Bank& bank) {
    std::string bytes(magic);
    for (const auto& prototype : bank.prototype) {
        append_word(bytes, prototype.count);
        for (int64_t coordinate : prototype.sum) {
            append_word(bytes, coordinate);
        }
    }
    write_once(path, bytes);
}

Bank load(const char* path) {
    auto stream = input(path);
    std::string header(sizeof(magic) - 1, '\0');
    stream.read(header.data(), header.size());
    require(header == magic, "Wrong state format");
    Bank bank{};
    for (auto& prototype : bank.prototype) {
        prototype.count = read_word(stream);
        for (auto& coordinate : prototype.sum) {
            coordinate = read_word(stream);
        }
    }
    require(stream.peek() == EOF && !stream.bad(), "State has trailing or unreadable bytes");
    validate(bank);
    return bank;
}

std::vector<Candidate> candidates(const char* path) {
    auto stream = input(path);
    std::vector<Candidate> rows;
    for (std::string line; std::getline(stream, line);) {
        Candidate row{};
        std::istringstream fields(line);
        for (int32_t& coordinate : row.coordinate) {
            require(static_cast<bool>(fields >> coordinate), "Expected 32 signed i32 coordinates");
        }
        require((fields >> std::ws).eof(), "Extra candidate fields");
        require(rows.size() < max_rows, "Too many candidates");
        rows.push_back(row);
    }
    require(stream.eof() && !stream.bad() && !rows.empty(), "Missing or unreadable candidates");
    return rows;
}

std::vector<int32_t> read_labels(const char* path, size_t count) {
    auto stream = input(path);
    std::vector<int32_t> labels;
    for (std::string line; std::getline(stream, line);) {
        std::istringstream fields(line);
        int32_t role;
        require((fields >> role) && role >= 0 && role < classes, "Expected role 0, 1 or 2");
        require((fields >> std::ws).eof(), "Extra role fields");
        require(labels.size() < count, "Too many labels");
        labels.push_back(role);
    }
    require(stream.eof() && !stream.bad() && labels.size() == count, "Wrong training label count");
    return labels;
}

void save_predictions(const char* path, const std::vector<Result>& results) {
    std::ostringstream bytes;
    for (const auto& result : results) {
        bytes << result.role;
        for (auto numerator : result.numerator) {
            bytes << '\t' << numerator;
        }
        bytes << '\n';
    }
    write_once(path, bytes.str());
}

void finite(const char* path) {
    // Distinct counts catch an implementation that compares raw sums.
    const std::vector<Candidate> train{{{-5}}, {{-3}}, {{0}}, {{2}}, {{4}}, {{8}}};
    const std::vector<int32_t> labels{0, 0, 1, 1, 1, 2};
    const Bank expected{{{2, {-8}}, {3, {6}}, {1, {8}}}};
    const auto fitted = fit(train, labels);
    save(path, fitted);
    const auto reloaded = load(path);
    for (int role = 0; role < classes; ++role) {
        require(fitted.prototype[role].count == expected.prototype[role].count &&
                reloaded.prototype[role].count == expected.prototype[role].count,
                "Finite fit/reload count mismatch");
        for (int axis = 0; axis < dimensions; ++axis) {
            require(fitted.prototype[role].sum[axis] == expected.prototype[role].sum[axis] &&
                    reloaded.prototype[role].sum[axis] == expected.prototype[role].sum[axis],
                    "Finite fit/reload sum mismatch");
        }
    }

    const std::vector<Candidate> held{{{-4}}, {{2}}, {{8}}, {{-1}}, {{5}}, {{4}}};
    const std::array<Result, 6> oracle{{
        {{0, 18, 12}, 0}, {{12, 0, 6}, 1}, {{24, 18, 0}, 2},
        {{6, 9, 9}, -1}, {{18, 9, 3}, -1}, {{16, 6, 4}, 1},
    }};
    const auto permuted = fit(train, {0, 0, 2, 2, 2, 1});
    const auto before = predict(held, fitted);
    const auto after = predict(held, reloaded);
    const auto permuted_results = predict(held, permuted);
    constexpr int swapped[] = {0, 2, 1};
    for (int role = 0; role < classes; ++role) {
        const auto& original = expected.prototype[swapped[role]];
        const auto& actual = permuted.prototype[role];
        require(actual.count == original.count, "Permuted label count mismatch");
        for (int axis = 0; axis < dimensions; ++axis) {
            require(actual.sum[axis] == original.sum[axis], "Permuted label sum mismatch");
        }
    }
    for (size_t row = 0; row < held.size(); ++row) {
        const auto host = classify(held[row], expected);
        const auto& permuted_result = permuted_results[row];
        const auto expected_role = oracle[row].role;
        require(permuted_result.role == (expected_role < 0 ? -1 : swapped[expected_role]),
                "Permuted prediction role mismatch");
        for (int role = 0; role < classes; ++role) {
            require(permuted_result.numerator[role] == oracle[row].numerator[swapped[role]],
                    "Permuted prediction distance mismatch");
        }
        for (const auto& result : {before[row], after[row], host}) {
            require(result.role == oracle[row].role, "Finite role mismatch");
            for (int role = 0; role < classes; ++role) {
                require(result.numerator[role] == oracle[row].numerator[role],
                        "Finite distance mismatch");
            }
        }
    }
    std::cout << "PASS 99 state words and 24 prediction words match the finite oracle"
              << " on host, CUDA and artifact replay, plus permuted labels\n";
}

int main(int argc, char** argv) {
    try {
        if (argc == 5 && std::string(argv[1]) == "fit") {
            const auto rows = candidates(argv[2]);
            save(argv[4], fit(rows, read_labels(argv[3], rows.size())));
        } else if (argc == 5 && std::string(argv[1]) == "predict") {
            save_predictions(argv[4], predict(candidates(argv[2]), load(argv[3])));
        } else if (argc == 3 && std::string(argv[1]) == "finite") {
            finite(argv[2]);
        } else {
            throw std::runtime_error(
                "Usage: d24-role fit FEATURES TRAIN_LABELS STATE | predict FEATURES STATE PREDICTIONS | finite STATE");
        }
        return 0;
    } catch (const std::exception& error) {
        std::cerr << error.what() << '\n';
        return 1;
    }
}
