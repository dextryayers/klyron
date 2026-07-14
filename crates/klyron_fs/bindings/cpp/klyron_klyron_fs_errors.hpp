#ifndef KLYRON_KLYRON_FS_ERRORS_HPP
#define KLYRON_KLYRON_FS_ERRORS_HPP

#include <stdexcept>
#include <string>

namespace klyron {
class FileSystemError : public std::runtime_error {
public:
    explicit FileSystemError(const std::string& msg) : std::runtime_error(msg) {}
};
}

#endif
