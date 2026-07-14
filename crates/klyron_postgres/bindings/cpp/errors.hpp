#ifndef KLYRON_POSTGRES_BINDINGS_ERRORS_HPP
#define KLYRON_POSTGRES_BINDINGS_ERRORS_HPP

#include <stdexcept>
#include <string>

namespace klyron {

class Klyron::PostgresException : public std::runtime_error {
public:
  explicit Klyron::PostgresException(const std::string& msg)
    : std::runtime_error(msg) {}
};

class NotFoundException : public Klyron::PostgresException {
public:
  explicit NotFoundException(const std::string& resource)
    : Klyron::PostgresException("Not found: " + resource) {}
};

class InvalidInputException : public Klyron::PostgresException {
public:
  explicit InvalidInputException(const std::string& detail)
    : Klyron::PostgresException("Invalid input: " + detail) {}
};

} // namespace klyron

#endif
