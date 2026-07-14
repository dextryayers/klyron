#pragma once
#include "types.hpp"
#include <memory>

namespace klyron_bundler {

class BundlerClient {
public:
    BundlerClient();
    explicit BundlerClient(const BundlerConfig& config);
    std::string version() const;
    BundlerConfig config() const;

private:
    BundlerConfig config_;
};

} // namespace
