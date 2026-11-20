// Reuse the exact fixed-weight CUDA mixture from the qualified A1 fit.
#define main d31_a1_fit_driver
#include "../../fit/device.cu"
#undef main

int main(int argc, char **argv) {
    try {
        constexpr Word candidates = 4095;
        if (argc != 4) throw std::runtime_error("usage: d31-client-mix ROWS WEIGHTS NEW-MASSES");
        FILE *file = fopen(argv[1], "rb");
        if (!file) throw std::runtime_error("open candidate rows");
        char magic[8];
        Word count = 0;
        std::vector<Row> host(candidates);
        if (fread(magic, 8, 1, file) != 1 || std::string(magic, 8) != "A1EM0001" ||
            fread(&count, 8, 1, file) != 1 || count != candidates ||
            fread(host.data(), sizeof(Row), count, file) != count ||
            fgetc(file) != EOF || ferror(file) || fclose(file) != 0) {
            throw std::runtime_error("malformed candidate rows");
        }
        for (const Row &r : host) {
            if (r.bucket >= buckets) throw std::runtime_error("invalid candidate bucket");
            for (Word m : r.mass) if (m > mass_one) throw std::runtime_error("invalid candidate mass");
        }
        file = fopen(argv[2], "rb");
        if (!file) throw std::runtime_error("open verified weights");
        uint32_t order = 0;
        char binding[32];
        std::vector<Word> weights(words);
        if (fread(magic, 8, 1, file) != 1 || std::string(magic, 8) != "UORS5W01" ||
            fread(&order, 4, 1, file) != 1 || order != 10 ||
            fread(binding, 32, 1, file) != 1 || fread(weights.data(), 8, words, file) != words ||
            fgetc(file) != EOF || ferror(file) || fclose(file) != 0) {
            throw std::runtime_error("invalid verified weights");
        }
        std::vector<Word> indices(count);
        for (Word i = 0; i < count; ++i) indices[i] = i;
        Row *on_rows;
        Word *on_weights, *on_indices, *on_masses;
        CUDA(cudaMalloc(&on_rows, count * sizeof(Row)));
        CUDA(cudaMalloc(&on_weights, words * sizeof(Word)));
        CUDA(cudaMalloc(&on_indices, count * sizeof(Word)));
        CUDA(cudaMalloc(&on_masses, count * sizeof(Word)));
        CUDA(cudaMemcpy(on_rows, host.data(), count * sizeof(Row), cudaMemcpyHostToDevice));
        CUDA(cudaMemcpy(on_weights, weights.data(), words * sizeof(Word), cudaMemcpyHostToDevice));
        CUDA(cudaMemcpy(on_indices, indices.data(), count * sizeof(Word), cudaMemcpyHostToDevice));
        probe_mass<<<(count + 63) / 64, 64>>>(on_rows, on_weights, on_indices, on_masses, unsigned(count));
        CUDA(cudaGetLastError());
        std::vector<Word> masses(count);
        CUDA(cudaMemcpy(masses.data(), on_masses, count * sizeof(Word), cudaMemcpyDeviceToHost));
        CUDA(cudaFree(on_rows));
        CUDA(cudaFree(on_weights));
        CUDA(cudaFree(on_indices));
        CUDA(cudaFree(on_masses));
        file = fopen(argv[3], "wbx");
        if (!file || fwrite(masses.data(), 8, count, file) != count || fclose(file) != 0) {
            throw std::runtime_error("could not seal candidate masses");
        }
        return 0;
    } catch (const std::exception &error) {
        fprintf(stderr, "%s\n", error.what());
        return 1;
    }
}
