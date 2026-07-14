#ifndef KLYRON_UPDATER_BINDINGS_ERRORS_HPP
#define KLYRON_UPDATER_BINDINGS_ERRORS_HPP

#include <stdexcept>
#include <string>

namespace klyron {

class Klyron::UpdaterException : public std::runtime_error {
public:
  explicit Klyron::UpdaterException(const std::string& msg)
    : std::runtime_error(msg) {}
};

class NotFoundException : public Klyron::UpdaterException {
public:
  explicit NotFoundException(const std::string& resource)
    : Klyron::UpdaterException("Not found: " + resource) {}
};

class InvalidInputException : public Klyron::UpdaterException {
public:
  explicit InvalidInputException(const std::string& detail)
    : Klyron::UpdaterException("Invalid input: " + detail) {}
};

} // namespace klyron

#endif
