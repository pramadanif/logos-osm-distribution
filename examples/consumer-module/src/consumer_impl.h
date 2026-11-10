#pragma once

#include <string>

#include "logos_module_context.h"

/// Template consumer of OSM map data. Every answer is the SDK module's JSON envelope
/// (`{"ok":true,"data":…}` or `{"ok":false,"code":…,"error":…}`), or this module's own
/// `{"ok":false,"code":"T001",…}` when the SDK module did not answer.
class ConsumerImpl : public LogosModuleContext
{
public:
    /// Newest hosted snapshot of `region`: CID, MD5, snapshot date, size, hosts.
    std::string mapFor(const std::string& region);
    /// Downloads `region` by CID and keeps it only if its MD5 matches the registry.
    std::string download(const std::string& region);
    /// The 72 regions with names, level and parent.
    std::string regions();
};
