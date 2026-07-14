#pragma once
#include "types.hpp"
#include <memory>

namespace klyron_formatter {

class FormatterClient {
public:
    FormatterClient();
    explicit FormatterClient(const FormatterConfig& config);
    std::string version() const;
    FormatterConfig config() const;

private:
    FormatterConfig config_;
};

} // namespace
