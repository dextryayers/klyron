#include "config.hpp"

namespace klyron::_compat {

CompatConfigBuilder& CompatConfigBuilder::with_version(const std::string& v) {
    config_.version = v;
    return *this;
}

CompatConfig CompatConfigBuilder::build() {
    return config_;
}

}
