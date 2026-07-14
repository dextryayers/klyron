#pragma once
#include "types.hpp"

namespace klyron::_compat {

class CompatConfigBuilder {
public:
    CompatConfigBuilder& with_version(const std::string& v);
    CompatConfig build();
private:
    CompatConfig config_;
};

}
