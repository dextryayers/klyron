#pragma once
#include "types.hpp"

namespace klyron::_template {

class TemplateClient {
public:
    explicit TemplateClient(const TemplateConfig& config);
    void execute();
private:
    TemplateConfig config_;
};

}
