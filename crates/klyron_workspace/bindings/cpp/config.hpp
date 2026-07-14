#pragma once
#include "types.hpp"

namespace klyron::_workspace {

class WorkspaceConfigBuilder {
public:
    WorkspaceConfigBuilder& with_version(const std::string& v);
    WorkspaceConfig build();
private:
    WorkspaceConfig config_;
};

}
