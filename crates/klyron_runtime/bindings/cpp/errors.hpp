#ifndef KLYRON_RUNTIME_BINDINGS_ERRORS_HPP
#define KLYRON_RUNTIME_BINDINGS_ERRORS_HPP

#include <stdexcept>
#include <string>

namespace klyron {

class Klyron::RuntimeException : public std::runtime_error {
public:
  explicit Klyron::RuntimeException(const std::string& msg)
    : std::runtime_error(msg) {}
};

class NotFoundException : public Klyron::RuntimeException {
public:
  explicit NotFoundException(const std::string& resource)
    : Klyron::RuntimeException("Not found: " + resource) {}
};

class InvalidInputException : public Klyron::RuntimeException {
public:
  explicit InvalidInputException(const std::string& detail)
    : Klyron::RuntimeException("Invalid input: " + detail) {}
};

} // namespace klyron

#endif
