#pragma once
#include "types.hpp"
#include "config.hpp"

namespace klyron::_config {

class ConfigBuilder {
public:
    ConfigBuilder();
    ConfigBuilder& set_config(const ConfigConfig& cfg);
    ConfigConfig config() const;
private:
    ConfigConfig config_;
};

}
