#ifndef KLYRON_SQLITE_BINDINGS_ERRORS_HPP
#define KLYRON_SQLITE_BINDINGS_ERRORS_HPP

#include <stdexcept>
#include <string>

namespace klyron {

class Klyron::SqliteException : public std::runtime_error {
public:
  explicit Klyron::SqliteException(const std::string& msg)
    : std::runtime_error(msg) {}
};

class NotFoundException : public Klyron::SqliteException {
public:
  explicit NotFoundException(const std::string& resource)
    : Klyron::SqliteException("Not found: " + resource) {}
};

class InvalidInputException : public Klyron::SqliteException {
public:
  explicit InvalidInputException(const std::string& detail)
    : Klyron::SqliteException("Invalid input: " + detail) {}
};

} // namespace klyron

#endif
