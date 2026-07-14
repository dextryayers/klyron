#pragma once
#include "types.hpp"

namespace klyron::_template {

class TemplateConfigBuilder {
public:
    TemplateConfigBuilder& with_version(const std::string& v);
    TemplateConfig build();
private:
    TemplateConfig config_;
};

}
