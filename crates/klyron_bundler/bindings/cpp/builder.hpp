#pragma once
#include "types.hpp"

namespace klyron_bundler {

class BundlerBuilder {
public:
    BundlerBuilder();
    BundlerBuilder& enabled(bool v);
    BundlerBuilder& verbose(bool v);
    BundlerConfig build();
private:
    BundlerConfig config_;
};

} // namespace
