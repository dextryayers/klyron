#pragma once
#include "types.hpp"

namespace klyron_registry {

class RegistryBuilder {
public:
    RegistryBuilder();
    RegistryBuilder& enabled(bool v);
    RegistryBuilder& verbose(bool v);
    RegistryConfig build();
private:
    RegistryConfig config_;
};

} // namespace
