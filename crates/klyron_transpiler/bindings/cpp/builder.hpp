#pragma once
#include "types.hpp"

namespace klyron_transpiler {

class TranspilerBuilder {
public:
    TranspilerBuilder();
    TranspilerBuilder& enabled(bool v);
    TranspilerBuilder& verbose(bool v);
    TranspilerConfig build();
private:
    TranspilerConfig config_;
};

} // namespace
