#pragma once
#include "types.hpp"
#include "config.hpp"

namespace klyron::_deploy {

class DeployBuilder {
public:
    DeployBuilder();
    DeployBuilder& set_config(const DeployConfig& cfg);
    DeployConfig config() const;
private:
    DeployConfig config_;
};

}
