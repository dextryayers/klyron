#pragma once
#include "types.hpp"

namespace klyron::_adapter {

class AdapterApi {
public:
    AdapterApi();
    void execute();
    static std::string version();
};

}
