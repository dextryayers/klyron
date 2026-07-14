#pragma once
#include "types.hpp"

namespace klyron::_shell {

class ShellClient {
public:
    explicit ShellClient(const ShellConfig& config);
    void execute();
private:
    ShellConfig config_;
};

}
