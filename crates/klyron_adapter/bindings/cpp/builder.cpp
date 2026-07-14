#include "builder.hpp"

namespace klyron::_adapter {

AdapterBuilder::AdapterBuilder() : config_{} {}

AdapterBuilder& AdapterBuilder::set_config(const AdapterConfig& cfg) {
    config_ = cfg;
    return *this;
}

AdapterConfig AdapterBuilder::config() const {
    return config_;
}

}
