#pragma once
#include "types.hpp"
#include <memory>

namespace klyron_pm {

class PmClient {
public:
    PmClient();
    explicit PmClient(const PmConfig& config);
    std::string version() const;
    PmConfig config() const;

private:
    PmConfig config_;
};

} // namespace
