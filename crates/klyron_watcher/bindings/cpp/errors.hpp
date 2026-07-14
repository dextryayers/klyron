#pragma once
#include <stdexcept>
#include <string>

namespace klyron_watcher {

class WatcherException : public std::runtime_error {
public:
    explicit WatcherException(const std::string& msg) : std::runtime_error(msg) {}
};

} // namespace
