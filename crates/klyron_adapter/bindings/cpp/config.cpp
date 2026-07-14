#include "config.hpp"

namespace klyron::_adapter {

AdapterConfigBuilder& AdapterConfigBuilder::with_version(const std::string& v) {
    config_.version = v;
    return *this;
}

AdapterConfig AdapterConfigBuilder::build() {
    return config_;
}

}
