#pragma once
#include "types.hpp"

namespace klyron_bench {

class BenchBuilder {
public:
    BenchBuilder();
    BenchBuilder& enabled(bool v);
    BenchBuilder& verbose(bool v);
    BenchConfig build();
private:
    BenchConfig config_;
};

} // namespace
