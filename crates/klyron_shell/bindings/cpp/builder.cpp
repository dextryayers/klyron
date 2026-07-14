#include "builder.hpp"

namespace klyron::_shell {

ShellBuilder::ShellBuilder() : config_{} {}

ShellBuilder& ShellBuilder::set_config(const ShellConfig& cfg) {
    config_ = cfg;
    return *this;
}

ShellConfig ShellBuilder::config() const {
    return config_;
}

}
