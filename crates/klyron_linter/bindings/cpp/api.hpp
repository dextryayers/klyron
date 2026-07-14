#pragma once
#include "types.hpp"
#include <memory>

namespace klyron_linter {

class LinterClient {
public:
    LinterClient();
    explicit LinterClient(const LinterConfig& config);
    std::string version() const;
    LinterConfig config() const;

private:
    LinterConfig config_;
};

} // namespace
