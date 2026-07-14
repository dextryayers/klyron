#include "klyron_klyron_process_api.hpp"
#include <string>

namespace klyron {

ProcessManagerApi::ProcessManagerApi() {}

std::string ProcessManagerApi::version() const {
    return "klyron_process 0.1.0";
}

bool ProcessManagerApi::ping() {
    return true;
}

}
