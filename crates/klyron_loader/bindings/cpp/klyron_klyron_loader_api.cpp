#include "klyron_klyron_loader_api.hpp"
#include <string>

namespace klyron {

ModuleLoaderApi::ModuleLoaderApi() {}

std::string ModuleLoaderApi::version() const {
    return "klyron_loader 0.1.0";
}

bool ModuleLoaderApi::ping() {
    return true;
}

}
