#include "builder.hpp"

namespace klyron::_compat {

CompatBuilder::CompatBuilder() : config_{} {}

CompatBuilder& CompatBuilder::set_config(const CompatConfig& cfg) {
    config_ = cfg;
    return *this;
}

CompatConfig CompatBuilder::config() const {
    return config_;
}

}
