#pragma once
#include "types.hpp"

namespace klyron::_compat {

class CompatApi {
public:
    CompatApi();
    void execute();
    static std::string version();
};

}
