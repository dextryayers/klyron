#ifndef KLYRON_ENGINE_BINDINGS_ERRORS_HPP
#define KLYRON_ENGINE_BINDINGS_ERRORS_HPP

#include <stdexcept>
#include <string>

namespace klyron {

class Klyron::EngineException : public std::runtime_error {
public:
  explicit Klyron::EngineException(const std::string& msg)
    : std::runtime_error(msg) {}
};

class NotFoundException : public Klyron::EngineException {
public:
  explicit NotFoundException(const std::string& resource)
    : Klyron::EngineException("Not found: " + resource) {}
};

class InvalidInputException : public Klyron::EngineException {
public:
  explicit InvalidInputException(const std::string& detail)
    : Klyron::EngineException("Invalid input: " + detail) {}
};

} // namespace klyron

#endif
