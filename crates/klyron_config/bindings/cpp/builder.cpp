#include "builder.hpp"

namespace klyron::_config {

ConfigBuilder::ConfigBuilder() : config_{} {}

ConfigBuilder& ConfigBuilder::set_config(const ConfigConfig& cfg) {
    config_ = cfg;
    return *this;
}

ConfigConfig ConfigBuilder::config() const {
    return config_;
}

}
