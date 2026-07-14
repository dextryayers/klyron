#pragma once
#include <stdexcept>
#include <string>

namespace klyron_napi {

class NapiException : public std::runtime_error {
public:
    explicit NapiException(const std::string& message) : std::runtime_error(message) {}
};

class ModuleNotFoundError : public NapiException {
public:
    explicit ModuleNotFoundError(const std::string& name)
        : NapiException("N-API module '" + name + "' not found") {}
};

class LoadFailedError : public NapiException {
public:
    LoadFailedError(const std::string& name, const std::string& reason)
        : NapiException("Failed to load '" + name + "': " + reason) {}
};

class VersionMismatchError : public NapiException {
public:
    VersionMismatchError(uint32_t expected, uint32_t got)
        : NapiException("Version mismatch: expected " + std::to_string(expected)
                        + ", got " + std::to_string(got)) {}
};

} // namespace klyron_napi
