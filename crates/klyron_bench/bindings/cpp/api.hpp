#pragma once
#include "types.hpp"
#include <memory>

namespace klyron_bench {

class BenchClient {
public:
    BenchClient();
    explicit BenchClient(const BenchConfig& config);
    std::string version() const;
    BenchConfig config() const;

private:
    BenchConfig config_;
};

} // namespace
