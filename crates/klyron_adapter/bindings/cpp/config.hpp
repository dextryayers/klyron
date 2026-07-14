#pragma once
#include "types.hpp"

namespace klyron::_adapter {

class AdapterConfigBuilder {
public:
    AdapterConfigBuilder& with_version(const std::string& v);
    AdapterConfig build();
private:
    AdapterConfig config_;
};

}
