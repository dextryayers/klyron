#include "builder.hpp"

namespace klyron::_workspace {

WorkspaceBuilder::WorkspaceBuilder() : config_{} {}

WorkspaceBuilder& WorkspaceBuilder::set_config(const WorkspaceConfig& cfg) {
    config_ = cfg;
    return *this;
}

WorkspaceConfig WorkspaceBuilder::config() const {
    return config_;
}

}
