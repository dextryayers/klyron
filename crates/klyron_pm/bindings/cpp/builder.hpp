#pragma once
#include "types.hpp"

namespace klyron_pm {

class PmBuilder {
public:
    PmBuilder();
    PmBuilder& enabled(bool v);
    PmBuilder& verbose(bool v);
    PmConfig build();
private:
    PmConfig config_;
};

} // namespace
