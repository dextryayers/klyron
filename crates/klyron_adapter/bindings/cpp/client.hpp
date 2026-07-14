#pragma once
#include "types.hpp"

namespace klyron::_adapter {

class AdapterClient {
public:
    explicit AdapterClient(const AdapterConfig& config);
    void execute();
private:
    AdapterConfig config_;
};

}
