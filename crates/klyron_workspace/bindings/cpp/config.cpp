#include "config.hpp"

namespace klyron::_workspace {

WorkspaceConfigBuilder& WorkspaceConfigBuilder::with_version(const std::string& v) {
    config_.version = v;
    return *this;
}

WorkspaceConfig WorkspaceConfigBuilder::build() {
    return config_;
}

}
