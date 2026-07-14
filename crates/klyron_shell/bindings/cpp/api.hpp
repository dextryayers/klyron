#pragma once
#include "types.hpp"

namespace klyron::_shell {

class ShellApi {
public:
    ShellApi();
    void execute();
    static std::string version();
};

}
