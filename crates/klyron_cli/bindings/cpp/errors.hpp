#ifndef KLYRON_CLI_BINDINGS_ERRORS_HPP
#define KLYRON_CLI_BINDINGS_ERRORS_HPP

#include <stdexcept>
#include <string>

namespace klyron {

class Klyron::CliException : public std::runtime_error {
public:
  explicit Klyron::CliException(const std::string& msg)
    : std::runtime_error(msg) {}
};

class NotFoundException : public Klyron::CliException {
public:
  explicit NotFoundException(const std::string& resource)
    : Klyron::CliException("Not found: " + resource) {}
};

class InvalidInputException : public Klyron::CliException {
public:
  explicit InvalidInputException(const std::string& detail)
    : Klyron::CliException("Invalid input: " + detail) {}
};

} // namespace klyron

#endif
