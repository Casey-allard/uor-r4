#include <cuda_runtime.h>

#include <array>
#include <charconv>
#include <cstdint>
#include <fstream>
#include <iostream>
#include <limits>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

#include <cerrno>
#include <fcntl.h>
#include <unistd.h>

namespace {

constexpr int dimensions = 32;
constexpr uint32_t max_bank = 8192;
constexpr size_t max_rows = 1000000;
constexpr int64_t absent = INT64_MAX;
constexpr char rows_magic[] = "D25ROWS1";
constexpr char bank_magic[] = "D25BANK1";

struct Exemplar {
    int16_t local[dimensions];
    int32_t context[dimensions];
    int8_t label;
};

struct Candidate {
    Exemplar feature;
    uint32_t row;
    uint32_t address;
    uint8_t narrow;
    uint8_t wide;
};

struct Group {
    uint32_t start;
    uint32_t end;
};

struct Distance {
    int64_t positive;
    int64_t negative;
};

enum Reason { pick, empty, missing, nonpositive, tie };

struct Selection {
    Reason reason;
    uint32_t address;
};

__host__ __device__ bool same_record(const Exemplar &left, const Exemplar &right) {
    if (left.label != right.label) return false;
    for (int i = 0; i < dimensions; ++i) {
        if (left.local[i] != right.local[i] || left.context[i] != right.context[i]) {
            return false;
        }
    }
    return true;
}

__global__ void fit_bank(const Candidate *rows, uint32_t count, Exemplar *bank,
                         uint32_t *bank_count) {
    uint32_t size = 0;
    for (uint32_t row = 0; row < count; ++row) {
        if (!rows[row].wide) continue;
        const Exemplar &exemplar = rows[row].feature;
        uint32_t index = 0;
        for (; index < size; ++index) {
            if (same_record(exemplar, bank[index])) break;
        }
        if (index == size) {
            if (size == max_bank) {
                *bank_count = max_bank + 1;
                return;
            }
            bank[size++] = exemplar;
        }
    }
    *bank_count = size;
}

__host__ __device__ int64_t absolute_difference(int64_t left, int64_t right) {
    return left >= right ? left - right : right - left;
}

__host__ __device__ Distance nearest(const Exemplar &feature, const Exemplar *bank,
                                     uint32_t count, bool contextual) {
    Distance result{absent, absent};
    for (uint32_t index = 0; index < count; ++index) {
        const Exemplar &exemplar = bank[index];
        int64_t distance = 0;
        for (int i = 0; i < dimensions; ++i) {
            distance += absolute_difference(feature.local[i], exemplar.local[i]);
            if (contextual) {
                distance += absolute_difference(feature.context[i], exemplar.context[i]);
            }
        }
        int64_t &best = exemplar.label == 1 ? result.positive : result.negative;
        if (distance < best) best = distance;
    }
    return result;
}

__host__ __device__ bool eligible(const Candidate &row, int arm) {
    return arm % 2 == 0 ? row.narrow : row.wide;
}

__host__ __device__ Selection select(const Candidate *rows, Group group,
                                     const Distance *distances, int arm) {
    int64_t best_margin = 0;
    uint32_t best_address = 0;
    bool any = false;
    bool tied = false;
    for (uint32_t index = group.start; index < group.end; ++index) {
        if (!eligible(rows[index], arm)) continue;
        any = true;
        const Distance &distance = distances[2 * index + arm / 2];
        if (distance.positive == absent || distance.negative == absent) return {missing, 0};
        const int64_t margin = distance.negative - distance.positive;
        if (margin > best_margin) {
            best_margin = margin;
            best_address = rows[index].address;
            tied = false;
        } else if (margin == best_margin && margin > 0) {
            tied = true;
        }
    }
    if (!any) return {empty, 0};
    if (best_margin == 0) return {nonpositive, 0};
    if (tied) return {tie, 0};
    return {pick, best_address};
}

__global__ void score_candidates(const Candidate *rows, uint32_t count,
                                 const Exemplar *bank, uint32_t bank_count,
                                 Distance *distances) {
    const uint32_t index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count) return;
    distances[2 * index] = nearest(rows[index].feature, bank, bank_count, false);
    distances[2 * index + 1] = nearest(rows[index].feature, bank, bank_count, true);
}

__global__ void select_groups(const Candidate *rows, const Group *groups,
                              uint32_t count, const Distance *distances,
                              Selection *selections) {
    const uint32_t index = blockIdx.x * blockDim.x + threadIdx.x;
    if (index >= count) return;
    for (int arm = 0; arm < 4; ++arm) {
        selections[4 * index + arm] = select(rows, groups[index], distances, arm);
    }
}

void require(bool condition, const char *message) {
    if (!condition) throw std::runtime_error(message);
}

void cuda_check(cudaError_t status) {
    if (status != cudaSuccess) throw std::runtime_error(cudaGetErrorString(status));
}

template <class T>
class DeviceBuffer {
    T *data_ = nullptr;

public:
    explicit DeviceBuffer(size_t count) {
        cuda_check(cudaMalloc(&data_, count * sizeof(T)));
    }

    ~DeviceBuffer() { cudaFree(data_); }
    DeviceBuffer(const DeviceBuffer &) = delete;
    DeviceBuffer &operator=(const DeviceBuffer &) = delete;

    T *data() { return data_; }

    void upload(const std::vector<T> &values) {
        cuda_check(cudaMemcpy(data_, values.data(), values.size() * sizeof(T),
                              cudaMemcpyHostToDevice));
    }

    std::vector<T> download(size_t count) {
        std::vector<T> values(count);
        cuda_check(cudaMemcpy(values.data(), data_, count * sizeof(T), cudaMemcpyDeviceToHost));
        return values;
    }
};

template <class T>
T read_integer(std::istream &input) {
    std::string field;
    require(static_cast<bool>(input >> field), "missing row field");
    T number{};
    const auto parsed = std::from_chars(field.data(), field.data() + field.size(), number);
    require(parsed.ec == std::errc{} && parsed.ptr == field.data() + field.size(),
            "invalid integer field");
    return number;
}

std::vector<Candidate> load_rows(const char *path, bool fitting) {
    std::ifstream input(path);
    std::string line;
    require(static_cast<bool>(std::getline(input, line)) && line == rows_magic,
            "invalid row packet header");
    std::vector<Candidate> rows;
    while (std::getline(input, line)) {
        require(rows.size() < max_rows, "too many candidate rows");
        std::istringstream fields(line);
        Candidate row{};
        row.row = read_integer<uint32_t>(fields);
        row.address = read_integer<uint32_t>(fields);
        row.narrow = read_integer<uint8_t>(fields);
        row.wide = read_integer<uint8_t>(fields);
        row.feature.label = read_integer<int8_t>(fields);
        require(row.narrow <= row.wide && row.wide <= 1, "invalid candidate mask");
        const int8_t label = row.feature.label;
        require(fitting && row.wide ? label == 1 || label == -1 : label == 0,
                "invalid label for packet mode and eligibility");
        for (int16_t &coordinate : row.feature.local) {
            coordinate = read_integer<int16_t>(fields);
            require(coordinate >= -48 && coordinate <= 48, "local coordinate out of range");
        }
        for (int32_t &coordinate : row.feature.context) {
            coordinate = read_integer<int32_t>(fields);
        }
        std::string extra;
        require(!(fields >> extra), "trailing candidate fields");
        if (!rows.empty()) {
            const auto &previous = rows.back();
            require(previous.row < row.row ||
                        (previous.row == row.row && previous.address < row.address),
                    "candidate rows or addresses out of order");
        }
        rows.push_back(row);
    }
    require(input.eof() && !rows.empty(), "empty or unreadable candidate packet");
    return rows;
}

void put_le(std::ostream &output, uint32_t bits, int bytes) {
    for (int i = 0; i < bytes; ++i) {
        output.put(static_cast<char>(bits & 255));
        bits >>= 8;
    }
}

uint32_t get_le(std::istream &input, int bytes) {
    uint32_t result = 0;
    for (int i = 0; i < bytes; ++i) {
        const int byte = input.get();
        require(byte != std::char_traits<char>::eof(), "truncated bank");
        result |= static_cast<uint32_t>(byte) << (8 * i);
    }
    return result;
}

int64_t signed_word(uint32_t bits, int width) {
    const int64_t modulus = int64_t{1} << width;
    return bits >= modulus / 2 ? static_cast<int64_t>(bits) - modulus : bits;
}

void validate_bank(const std::vector<Exemplar> &bank) {
    bool positive = false;
    bool negative = false;
    for (const auto &row : bank) {
        positive |= row.label == 1;
        negative |= row.label == -1;
    }
    require(positive && negative, "bank must contain both labels");
}

std::string bank_bytes(const std::vector<Exemplar> &bank) {
    std::ostringstream output;
    output.write(bank_magic, sizeof(bank_magic) - 1);
    put_le(output, static_cast<uint32_t>(bank.size()), 4);
    for (const auto &row : bank) {
        put_le(output, static_cast<uint8_t>(row.label), 1);
        for (const auto coordinate : row.local) put_le(output, static_cast<uint16_t>(coordinate), 2);
        for (const auto coordinate : row.context) put_le(output, static_cast<uint32_t>(coordinate), 4);
    }
    return output.str();
}

std::vector<Exemplar> load_bank(std::istream &input) {
    std::array<char, sizeof(bank_magic) - 1> header{};
    require(static_cast<bool>(input.read(header.data(), header.size())) &&
                std::string(header.data(), header.size()) == bank_magic,
            "invalid bank header");
    const uint32_t count = get_le(input, 4);
    require(count >= 2 && count <= max_bank, "invalid bank count");
    std::vector<Exemplar> bank(count);
    for (auto &row : bank) {
        row.label = static_cast<int8_t>(signed_word(get_le(input, 1), 8));
        require(row.label == 1 || row.label == -1, "invalid bank label");
        for (auto &coordinate : row.local) {
            coordinate = static_cast<int16_t>(signed_word(get_le(input, 2), 16));
            require(coordinate >= -48 && coordinate <= 48, "invalid bank local coordinate");
        }
        for (auto &coordinate : row.context) {
            coordinate = static_cast<int32_t>(signed_word(get_le(input, 4), 32));
        }
    }
    require(input.peek() == std::char_traits<char>::eof() && input.eof() && !input.bad(),
            "bank has trailing bytes or read error");
    validate_bank(bank);
    return bank;
}

void save_fresh(const char *path, const std::string &bytes) {
    const int descriptor = open(path, O_WRONLY | O_CREAT | O_EXCL, 0644);
    require(descriptor >= 0, "cannot create fresh output");
    size_t position = 0;
    while (position < bytes.size()) {
        const ssize_t count = write(descriptor, bytes.data() + position, bytes.size() - position);
        if (count < 0 && errno == EINTR) continue;
        if (count <= 0) break;
        position += count;
    }
    const int status = close(descriptor);
    require(status == 0 && position == bytes.size(), "could not write full output");
}

std::vector<Exemplar> fit(const std::vector<Candidate> &rows) {
    DeviceBuffer<Candidate> device_rows(rows.size());
    DeviceBuffer<Exemplar> device_bank(max_bank);
    DeviceBuffer<uint32_t> device_count(1);
    device_rows.upload(rows);
    fit_bank<<<1, 1>>>(device_rows.data(), rows.size(), device_bank.data(), device_count.data());
    cuda_check(cudaGetLastError());
    const uint32_t count = device_count.download(1).front();
    require(count <= max_bank, "fit exceeds bank capacity");
    return device_bank.download(count);
}

std::vector<Group> group_rows(const std::vector<Candidate> &rows) {
    std::vector<Group> groups;
    uint32_t start = 0;
    for (uint32_t end = 1; end <= rows.size(); ++end) {
        if (end == rows.size() || rows[end].row != rows[start].row) {
            groups.push_back({start, end});
            start = end;
        }
    }
    return groups;
}

struct Predictions {
    std::vector<Distance> distances;
    std::vector<Selection> selections;
};

Predictions score(const std::vector<Candidate> &rows, const std::vector<Group> &groups,
                  const std::vector<Exemplar> &bank) {
    DeviceBuffer<Candidate> device_rows(rows.size());
    DeviceBuffer<Group> device_groups(groups.size());
    DeviceBuffer<Exemplar> device_bank(bank.size());
    DeviceBuffer<Distance> device_distances(2 * rows.size());
    DeviceBuffer<Selection> device_selections(4 * groups.size());
    device_rows.upload(rows);
    device_groups.upload(groups);
    device_bank.upload(bank);
    constexpr uint32_t threads = 128;
    score_candidates<<<(rows.size() + threads - 1) / threads, threads>>>(
        device_rows.data(), rows.size(), device_bank.data(), bank.size(), device_distances.data());
    cuda_check(cudaGetLastError());
    select_groups<<<(groups.size() + threads - 1) / threads, threads>>>(
        device_rows.data(), device_groups.data(), groups.size(), device_distances.data(),
        device_selections.data());
    cuda_check(cudaGetLastError());
    return {device_distances.download(2 * rows.size()), device_selections.download(4 * groups.size())};
}

std::string field(int64_t value) {
    return value == absent ? "NA" : std::to_string(value);
}

void predict(const char *rows_path, const char *bank_path, const char *trace_path,
             const char *results_path) {
    const auto rows = load_rows(rows_path, false);
    std::ifstream input(bank_path, std::ios::binary);
    const auto bank = load_bank(input);
    const auto groups = group_rows(rows);
    const auto predictions = score(rows, groups, bank);
    std::ostringstream trace;
    for (size_t index = 0; index < rows.size(); ++index) {
        const Candidate &row = rows[index];
        for (int arm = 0; arm < 4; ++arm) {
            const Distance &distance = predictions.distances[2 * index + arm / 2];
            const bool complete = distance.positive != absent && distance.negative != absent;
            trace << row.row << '\t' << row.address << '\t' << arm << '\t' << eligible(row, arm)
                  << '\t' << field(distance.positive) << '\t' << field(distance.negative) << '\t'
                  << field(complete ? distance.negative - distance.positive : absent) << '\n';
        }
    }
    constexpr const char *reasons[]{"pick", "empty", "missing", "nonpositive", "tie"};
    std::ostringstream results;
    for (size_t index = 0; index < groups.size(); ++index) {
        for (int arm = 0; arm < 4; ++arm) {
            const Selection &choice = predictions.selections[4 * index + arm];
            results << rows[groups[index].start].row << '\t' << arm << '\t'
                    << reasons[choice.reason] << '\t'
                    << field(choice.reason == pick ? choice.address : absent) << '\n';
        }
    }
    save_fresh(trace_path, trace.str());
    save_fresh(results_path, results.str());
}

Candidate example(uint32_t row, uint32_t address, int16_t local, int32_t context,
                  int8_t label, bool narrow, bool wide = true) {
    Candidate result{};
    result.row = row;
    result.address = address;
    result.feature.local[0] = local;
    result.feature.context[0] = context;
    result.feature.label = label;
    result.narrow = narrow;
    result.wide = wide;
    return result;
}

void verify_fixture(const std::vector<Candidate> &rows, const std::vector<Group> &groups,
                    const std::vector<Exemplar> &bank, const Predictions &actual) {
    std::vector<Distance> expected;
    for (const auto &row : rows) {
        expected.push_back(nearest(row.feature, bank.data(), bank.size(), false));
        expected.push_back(nearest(row.feature, bank.data(), bank.size(), true));
    }
    for (size_t index = 0; index < expected.size(); ++index) {
        require(expected[index].positive == actual.distances[index].positive &&
                    expected[index].negative == actual.distances[index].negative,
                "finite raw distance replay mismatch");
    }
    for (size_t index = 0; index < groups.size(); ++index) {
        for (int arm = 0; arm < 4; ++arm) {
            const auto reference = select(rows.data(), groups[index], expected.data(), arm);
            const auto result = actual.selections[4 * index + arm];
            require(reference.reason == result.reason && reference.address == result.address,
                    "finite selection replay mismatch");
        }
    }
}

void finite(const char *bank_path) {
    const auto positive = example(0, 5, 24, INT32_MAX, 1, false);
    const auto negative = example(0, 2, -24, INT32_MIN, -1, true);
    const auto contextual_negative = example(1, 3, 24, INT32_MIN, -1, true);
    const std::vector<Candidate> train{negative, positive, contextual_negative, positive};
    const auto bank = fit(train);
    require(bank.size() == 3 && same_record(bank[0], negative.feature) &&
                same_record(bank[1], positive.feature) &&
                same_record(bank[2], contextual_negative.feature),
            "finite ordered bank mismatch");

    const std::string bytes = bank_bytes(bank);
    std::istringstream serialized(bytes);
    const auto reloaded = load_bank(serialized);
    require(bank_bytes(reloaded) == bytes && bytes.size() == 12 + 3 * 193,
            "finite serialization replay mismatch");

    const std::vector<Candidate> rows{
        example(0, 2, -24, INT32_MIN, 0, true),
        example(0, 3, 24, INT32_MIN, 0, true),
        example(0, 5, 24, INT32_MAX, 0, false),
        example(1, 0, 0, 0, 0, false, false),
        example(2, 2, 24, INT32_MAX, 0, true),
        example(2, 3, 24, INT32_MAX, 0, true),
        example(3, 2, 24, INT32_MAX, 0, true),
    };
    const auto groups = group_rows(rows);
    const auto original = score(rows, groups, reloaded);
    verify_fixture(rows, groups, reloaded, original);
    require(original.selections[0].reason == nonpositive &&
                original.selections[1].reason == nonpositive &&
                original.selections[2].reason == nonpositive &&
                original.selections[3].reason == pick && original.selections[3].address == 5 &&
                original.selections[7].reason == empty && original.selections[11].reason == tie &&
                original.selections[14].reason == pick && original.selections[14].address == 2 &&
                original.distances[5].positive == 0 &&
                original.distances[5].negative == int64_t{UINT32_MAX},
            "finite planted calls or extreme-coordinate distance mismatch");

    const std::vector<Exemplar> incomplete{positive.feature};
    const auto absent_negative = score(rows, groups, incomplete);
    verify_fixture(rows, groups, incomplete, absent_negative);
    require(absent_negative.selections[3].reason == missing &&
                absent_negative.distances[5].negative == absent,
            "finite missing class must abstain");

    auto swapped_train = train;
    for (auto &row : swapped_train) row.feature.label = -row.feature.label;
    const auto swapped = fit(swapped_train);
    require(swapped.size() == bank.size(), "finite label swap bank size mismatch");
    for (size_t index = 0; index < bank.size(); ++index) {
        auto expected = bank[index];
        expected.label = -expected.label;
        require(same_record(expected, swapped[index]), "finite label swap changed bank features");
    }
    const auto reversed = score(rows, groups, swapped);
    verify_fixture(rows, groups, swapped, reversed);
    require(reversed.selections[3].reason == pick && reversed.selections[3].address == 2 && reversed.selections[15].reason == nonpositive &&
                reversed.distances[5].positive == original.distances[5].negative &&
                reversed.distances[5].negative == original.distances[5].positive,
            "finite label swap did not reverse distances and abstain");

    for (std::string malformed : {bytes.substr(0, bytes.size() - 1), bytes + "x"}) {
        bool refused = false;
        try {
            std::istringstream input(malformed);
            load_bank(input);
        } catch (const std::runtime_error &) {
            refused = true;
        }
        require(refused, "finite malformed bank accepted");
    }
    save_fresh(bank_path, bytes);
    std::cout << "PASS D25 finite ordered fit, byte replay, four arms, missing class, label swap,"
                 " corrupt banks, positive tie, empty group and +5 third candidate\n";
}

} // namespace

int main(int argc, char **argv) {
    try {
        if (argc == 3 && std::string(argv[1]) == "finite") {
            finite(argv[2]);
        } else if (argc == 4 && std::string(argv[1]) == "fit") {
            const auto rows = load_rows(argv[2], true);
            bool positive = false;
            bool negative = false;
            for (const auto &row : rows) {
                positive |= row.feature.label == 1;
                negative |= row.feature.label == -1;
            }
            require(positive && negative, "fit input must contain both labels");
            const auto bank = fit(rows);
            validate_bank(bank);
            save_fresh(argv[3], bank_bytes(bank));
        } else if (argc == 6 && std::string(argv[1]) == "predict") {
            predict(argv[2], argv[3], argv[4], argv[5]);
        } else {
            throw std::runtime_error("usage: d25-source finite BANK | fit ROWS BANK | "
                                     "predict ROWS BANK TRACE RESULTS");
        }
    } catch (const std::exception &error) {
        std::cerr << error.what() << '\n';
        return 1;
    }
}
