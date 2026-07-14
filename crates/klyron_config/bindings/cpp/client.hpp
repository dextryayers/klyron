#pragma once
#include "types.hpp"

namespace klyron::_config {

class ConfigClient {
public:
    explicit ConfigClient(const ConfigConfig& config);
    void execute();
private:
    ConfigConfig config_;
};

}
