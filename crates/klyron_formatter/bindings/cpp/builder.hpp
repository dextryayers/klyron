#pragma once
#include "types.hpp"

namespace klyron_formatter {

class FormatterBuilder {
public:
    FormatterBuilder();
    FormatterBuilder& enabled(bool v);
    FormatterBuilder& verbose(bool v);
    FormatterConfig build();
private:
    FormatterConfig config_;
};

} // namespace
