#ifndef KLYRON_AI_BINDINGS_ERRORS_HPP
#define KLYRON_AI_BINDINGS_ERRORS_HPP

#include <stdexcept>
#include <string>

namespace klyron {

class Klyron::AiException : public std::runtime_error {
public:
  explicit Klyron::AiException(const std::string& msg)
    : std::runtime_error(msg) {}
};

class NotFoundException : public Klyron::AiException {
public:
  explicit NotFoundException(const std::string& resource)
    : Klyron::AiException("Not found: " + resource) {}
};

class InvalidInputException : public Klyron::AiException {
public:
  explicit InvalidInputException(const std::string& detail)
    : Klyron::AiException("Invalid input: " + detail) {}
};

} // namespace klyron

#endif
