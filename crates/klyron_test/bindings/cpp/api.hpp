#pragma once
#include "types.hpp"
#include <memory>

namespace klyron_test {

class TestClient {
public:
    TestClient();
    explicit TestClient(const TestConfig& config);
    std::string version() const;
    TestConfig config() const;

private:
    TestConfig config_;
};

} // namespace
