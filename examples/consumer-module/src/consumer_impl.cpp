#include "consumer_impl.h"

#include "logos_sdk.h"

namespace {

std::string unavailable(const logos::CallError& err)
{
    // err.message is shown to people as plain text; keep it out of the JSON structure.
    std::string m;
    for (char c : err.message)
        m += (c == '"' || c == '\\' || static_cast<unsigned char>(c) < 0x20) ? ' ' : c;
    return "{\"ok\":false,\"code\":\"T001\",\"error\":\"pramadani_osm_sdk did not answer: " + err.code + " " + m + "\"}";
}

} // namespace

std::string ConsumerImpl::mapFor(const std::string& region)
{
    logos::CallError err;
    std::string out = modules().pramadani_osm_sdk.resolve(region, &err, 120000);
    return err.ok() ? out : unavailable(err);
}

std::string ConsumerImpl::download(const std::string& region)
{
    logos::CallError err;
    // Country files can be several GB; allow hours.
    std::string out = modules().pramadani_osm_sdk.fetch(region, &err, 6 * 60 * 60 * 1000);
    return err.ok() ? out : unavailable(err);
}

std::string ConsumerImpl::regions()
{
    logos::CallError err;
    std::string out = modules().pramadani_osm_sdk.discover(&err, 120000);
    return err.ok() ? out : unavailable(err);
}
