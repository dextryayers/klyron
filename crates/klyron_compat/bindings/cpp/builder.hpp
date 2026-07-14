#pragma once
#include "types.hpp"
#include "config.hpp"

namespace klyron::_compat {

class CompatBuilder {
public:
    CompatBuilder();
    CompatBuilder& set_config(const CompatConfig& cfg);
    CompatConfig config() const;
private:
    CompatConfig config_;
};

}
