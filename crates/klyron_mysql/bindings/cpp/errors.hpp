#ifndef KLYRON_MYSQL_BINDINGS_ERRORS_HPP
#define KLYRON_MYSQL_BINDINGS_ERRORS_HPP

#include <stdexcept>
#include <string>

namespace klyron {

class Klyron::MysqlException : public std::runtime_error {
public:
  explicit Klyron::MysqlException(const std::string& msg)
    : std::runtime_error(msg) {}
};

class NotFoundException : public Klyron::MysqlException {
public:
  explicit NotFoundException(const std::string& resource)
    : Klyron::MysqlException("Not found: " + resource) {}
};

class InvalidInputException : public Klyron::MysqlException {
public:
  explicit InvalidInputException(const std::string& detail)
    : Klyron::MysqlException("Invalid input: " + detail) {}
};

} // namespace klyron

#endif
