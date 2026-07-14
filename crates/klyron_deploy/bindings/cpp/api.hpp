#pragma once
#include "types.hpp"

namespace klyron::_deploy {

class DeployApi {
public:
    DeployApi();
    void execute();
    static std::string version();
};

}
