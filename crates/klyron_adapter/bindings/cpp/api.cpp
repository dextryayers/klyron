#include "api.hpp"

namespace klyron::_adapter {

AdapterApi::AdapterApi() {}

void AdapterApi::execute() {
}

std::string AdapterApi::version() {
    return "klyron_adapter@0.1.0";
}

}
