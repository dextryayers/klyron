#ifndef KLYRON_KLYRON_NODE_ERRORS_HPP
#define KLYRON_KLYRON_NODE_ERRORS_HPP

#include <stdexcept>
#include <string>

namespace klyron {
class NodeGlobalsError : public std::runtime_error {
public:
    explicit NodeGlobalsError(const std::string& msg) : std::runtime_error(msg) {}
};
}

#endif
