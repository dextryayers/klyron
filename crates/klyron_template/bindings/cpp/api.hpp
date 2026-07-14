#pragma once
#include "types.hpp"

namespace klyron::_template {

class TemplateApi {
public:
    TemplateApi();
    void execute();
    static std::string version();
};

}
