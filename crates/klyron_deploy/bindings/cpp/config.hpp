#pragma once
#include "types.hpp"

namespace klyron::_deploy {

class DeployConfigBuilder {
public:
    DeployConfigBuilder& with_version(const std::string& v);
    DeployConfig build();
private:
    DeployConfig config_;
};

}
