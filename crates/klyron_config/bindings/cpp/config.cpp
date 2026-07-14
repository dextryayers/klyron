#include "config.hpp"

namespace klyron::_config {

ConfigConfigBuilder& ConfigConfigBuilder::with_version(const std::string& v) {
    config_.version = v;
    return *this;
}

ConfigConfig ConfigConfigBuilder::build() {
    return config_;
}

}
