#pragma once
#include "types.hpp"

namespace klyron::_compat {

class CompatClient {
public:
    explicit CompatClient(const CompatConfig& config);
    void execute();
private:
    CompatConfig config_;
};

}
