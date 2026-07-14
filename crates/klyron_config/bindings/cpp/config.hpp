#pragma once
#include "types.hpp"

namespace klyron::_config {

class ConfigConfigBuilder {
public:
    ConfigConfigBuilder& with_version(const std::string& v);
    ConfigConfig build();
private:
    ConfigConfig config_;
};

}
