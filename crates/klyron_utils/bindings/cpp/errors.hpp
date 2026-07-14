#ifndef KLYRON_UTILS_BINDINGS_ERRORS_HPP
#define KLYRON_UTILS_BINDINGS_ERRORS_HPP

#include <stdexcept>
#include <string>

namespace klyron {

class Klyron::UtilsException : public std::runtime_error {
public:
  explicit Klyron::UtilsException(const std::string& msg)
    : std::runtime_error(msg) {}
};

class NotFoundException : public Klyron::UtilsException {
public:
  explicit NotFoundException(const std::string& resource)
    : Klyron::UtilsException("Not found: " + resource) {}
};

class InvalidInputException : public Klyron::UtilsException {
public:
  explicit InvalidInputException(const std::string& detail)
    : Klyron::UtilsException("Invalid input: " + detail) {}
};

} // namespace klyron

#endif
