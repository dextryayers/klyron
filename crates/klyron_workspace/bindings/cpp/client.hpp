#pragma once
#include "types.hpp"

namespace klyron::_workspace {

class WorkspaceClient {
public:
    explicit WorkspaceClient(const WorkspaceConfig& config);
    void execute();
private:
    WorkspaceConfig config_;
};

}
