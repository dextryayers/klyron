#pragma once
#include "types.hpp"

namespace klyron::_config {

class ConfigApi {
public:
    ConfigApi();
    void execute();
    static std::string version();
};

}
