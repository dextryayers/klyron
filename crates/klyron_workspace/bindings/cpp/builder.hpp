#pragma once
#include "types.hpp"
#include "config.hpp"

namespace klyron::_workspace {

class WorkspaceBuilder {
public:
    WorkspaceBuilder();
    WorkspaceBuilder& set_config(const WorkspaceConfig& cfg);
    WorkspaceConfig config() const;
private:
    WorkspaceConfig config_;
};

}
