#pragma once
#include "types.hpp"

namespace klyron::_workspace {

class WorkspaceApi {
public:
    WorkspaceApi();
    void execute();
    static std::string version();
};

}
