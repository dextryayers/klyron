#pragma once
#include "types.hpp"

namespace klyron::_deploy {

class DeployClient {
public:
    explicit DeployClient(const DeployConfig& config);
    void execute();
private:
    DeployConfig config_;
};

}
