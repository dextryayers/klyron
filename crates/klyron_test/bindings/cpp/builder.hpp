#pragma once
#include "types.hpp"

namespace klyron_test {

class TestBuilder {
public:
    TestBuilder();
    TestBuilder& enabled(bool v);
    TestBuilder& verbose(bool v);
    TestConfig build();
private:
    TestConfig config_;
};

} // namespace
