#pragma once
#include "types.hpp"
#include "config.hpp"

namespace klyron::_shell {

class ShellBuilder {
public:
    ShellBuilder();
    ShellBuilder& set_config(const ShellConfig& cfg);
    ShellConfig config() const;
private:
    ShellConfig config_;
};

}
