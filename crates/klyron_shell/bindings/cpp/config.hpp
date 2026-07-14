#pragma once
#include "types.hpp"

namespace klyron::_shell {

class ShellConfigBuilder {
public:
    ShellConfigBuilder& with_version(const std::string& v);
    ShellConfig build();
private:
    ShellConfig config_;
};

}
