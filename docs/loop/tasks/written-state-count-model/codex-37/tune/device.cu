// Frozen forward only. Reuse the fitter's exact initialized-state probe kernel.
#define main d37_fit_entry
#include "../fit/device.cu"
#undef main

__global__ void score_bits(const Word *masses, Word *scored, unsigned count) {
    unsigned i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= count) return;
    scored[3 * i] = masses[2 * i];
    scored[3 * i + 1] = masses[2 * i + 1];
    scored[3 * i + 2] = bits_q12(masses[2 * i + 1]);
}

int main() {
    try {
        constexpr unsigned batch = 8192;
        Word buckets = 0;
        if (fread(&buckets, sizeof(Word), 1, stdin) != 1 ||
            (buckets != 528 && buckets != 880)) {
            throw std::runtime_error("wrong frozen bucket count");
        }
        std::vector<Word> weights(buckets * experts);
        std::vector<Word> ordinals(batch), scored(batch * 3);
        std::vector<Row> rows(batch);
        if (fread(weights.data(), sizeof(Word), weights.size(), stdin) != weights.size()) {
            throw std::runtime_error("truncated frozen weights");
        }
        Word *device_weights, *device_ordinals, *device_masses, *device_scored;
        Row *device_rows;
        CUDA(cudaMalloc(&device_weights, weights.size() * sizeof(Word)));
        CUDA(cudaMalloc(&device_ordinals, batch * sizeof(Word)));
        CUDA(cudaMalloc(&device_masses, batch * 2 * sizeof(Word)));
        CUDA(cudaMalloc(&device_scored, batch * 3 * sizeof(Word)));
        CUDA(cudaMalloc(&device_rows, batch * sizeof(Row)));
        CUDA(cudaMemcpy(device_weights, weights.data(), weights.size() * sizeof(Word),
                        cudaMemcpyHostToDevice));
        for (unsigned i = 0; i < batch; ++i) ordinals[i] = i;
        CUDA(cudaMemcpy(device_ordinals, ordinals.data(), batch * sizeof(Word),
                        cudaMemcpyHostToDevice));
        for (;;) {
            Word n = 0;
            if (fread(&n, 1, sizeof(Word), stdin) != sizeof(Word) || n > batch) {
                throw std::runtime_error("invalid frozen score batch");
            }
            if (n == 0) {
                if (fgetc(stdin) != EOF || ferror(stdin))
                    throw std::runtime_error("trailing frozen score input");
                break;
            }
            if (fread(rows.data(), sizeof(Row), n, stdin) != n)
                throw std::runtime_error("truncated frozen rows");
            for (Word i = 0; i < n; ++i) {
                if (rows[i].bucket >= buckets)
                    throw std::runtime_error("frozen row outside bucket domain");
                for (Word m : rows[i].mass)
                    if (m > mass_one) throw std::runtime_error("mass outside Q48 domain");
            }
            CUDA(cudaMemcpy(device_rows, rows.data(), n * sizeof(Row), cudaMemcpyHostToDevice));
            probe_mass<<<(n + 63) / 64, 64>>>(device_rows, device_weights, device_ordinals,
                                            device_masses, unsigned(n));
            CUDA(cudaGetLastError());
            score_bits<<<(n + 63) / 64, 64>>>(device_masses, device_scored, unsigned(n));
            CUDA(cudaGetLastError());
            CUDA(cudaMemcpy(scored.data(), device_scored, 3 * n * sizeof(Word),
                            cudaMemcpyDeviceToHost));
            if (fwrite(scored.data(), 3 * sizeof(Word), n, stdout) != n || fflush(stdout))
                throw std::runtime_error("failed frozen mass write");
        }
        CUDA(cudaFree(device_rows));
        CUDA(cudaFree(device_masses));
        CUDA(cudaFree(device_scored));
        CUDA(cudaFree(device_ordinals));
        CUDA(cudaFree(device_weights));
        return 0;
    } catch (const std::exception &e) {
        fprintf(stderr, "%s\n", e.what());
        return 1;
    }
}
