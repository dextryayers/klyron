#pragma once
#include "types.hpp"

namespace klyron_linter {

class LinterBuilder {
public:
    LinterBuilder();
    LinterBuilder& enabled(bool v);
    LinterBuilder& verbose(bool v);
    LinterConfig build();
private:
    LinterConfig config_;
};

} // namespace
