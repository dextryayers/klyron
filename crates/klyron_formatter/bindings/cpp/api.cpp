#include "api.hpp"

namespace klyron_formatter {

FormatterClient::FormatterClient() : config_() {}
FormatterClient::FormatterClient(const FormatterConfig& config) : config_(config) {}
std::string FormatterClient::version() const { return "1.0.0"; }
FormatterConfig FormatterClient::config() const { return config_; }

} // namespace
