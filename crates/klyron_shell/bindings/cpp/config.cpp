#include "config.hpp"

namespace klyron::_shell {

ShellConfigBuilder& ShellConfigBuilder::with_version(const std::string& v) {
    config_.version = v;
    return *this;
}

ShellConfig ShellConfigBuilder::build() {
    return config_;
}

}
