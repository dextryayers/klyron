#pragma once
#include "types.hpp"
#include "config.hpp"

namespace klyron::_adapter {

class AdapterBuilder {
public:
    AdapterBuilder();
    AdapterBuilder& set_config(const AdapterConfig& cfg);
    AdapterConfig config() const;
private:
    AdapterConfig config_;
};

}
