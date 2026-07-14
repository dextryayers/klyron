#pragma once
#include "types.hpp"
#include <memory>

namespace klyron_transpiler {

class TranspilerClient {
public:
    TranspilerClient();
    explicit TranspilerClient(const TranspilerConfig& config);
    std::string version() const;
    TranspilerConfig config() const;

private:
    TranspilerConfig config_;
};

} // namespace
