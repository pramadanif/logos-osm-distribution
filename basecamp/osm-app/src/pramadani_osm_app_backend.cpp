#include "pramadani_osm_app_backend.h"

#include "logos_sdk.h"

namespace {

// Downloads and uploads of country files can take hours on a slow link.
constexpr int kLongMs = 6 * 60 * 60 * 1000;
// A write waits up to 600 s for the read-back inside the module; leave room for the fee check.
constexpr int kWriteMs = 12 * 60 * 1000;
constexpr int kReadMs = 2 * 60 * 1000;

} // namespace

QString PramadaniOsmAppBackend::call(QString method, QString arg)
{
    const QString id = m_jobs.start(method, arg);
    publish();

    auto done = [this, id](QString answer) { finish(id, answer); };
    auto& osm = modules().pramadani_osm_distribution;
    if (method == "version") osm.versionAsync(done, Timeout(kReadMs));
    else if (method == "settings") osm.settingsAsync(done, Timeout(kReadMs));
    else if (method == "configure") osm.configureAsync(arg, done, Timeout(kReadMs));
    else if (method == "discover") osm.discoverAsync(done, Timeout(kReadMs));
    else if (method == "resolve") osm.resolveAsync(arg, done, Timeout(kReadMs));
    else if (method == "hosted_status") osm.hosted_statusAsync(done, Timeout(kReadMs));
    // One HEAD request per region to Geofabrik: all 72 can take minutes on a slow day.
    else if (method == "check_updates") osm.check_updatesAsync(arg, done, Timeout(kLongMs));
    else if (method == "fetch") osm.fetchAsync(arg, done, Timeout(kLongMs));
    else if (method == "host") osm.hostAsync(arg, done, Timeout(kLongMs));
    else if (method == "host_batch") osm.host_batchAsync(arg, done, Timeout(kLongMs));
    else if (method == "register_batch") osm.register_batchAsync(arg, done, Timeout(kWriteMs));
    else if (method == "lookup") osm.lookupAsync(arg, done, Timeout(kReadMs));
    else if (method == "status") osm.statusAsync(done, Timeout(kReadMs));
    else if (method == "import_file") osm.import_fileAsync(arg, done, Timeout(kLongMs));
    else if (method == "register_entry") osm.register_entryAsync(arg, done, Timeout(kWriteMs));
    else if (method == "unhost") osm.unhostAsync(arg, done, Timeout(kWriteMs));
    else finish(id, JobTable::failure("A002", QStringLiteral("UnknownAction: %1").arg(method)));
    return id;
}

void PramadaniOsmAppBackend::finish(const QString& id, const QString& answer)
{
    m_jobs.finish(id, answer);
    publish();
}

void PramadaniOsmAppBackend::clearFinished()
{
    m_jobs.clearFinished();
    publish();
}

void PramadaniOsmAppBackend::publish()
{
    setJobs(m_jobs.json());
}
