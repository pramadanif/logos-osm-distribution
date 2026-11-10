#include "pramadani_osm_consumer_app_backend.h"

#include "logos_sdk.h"

void PramadaniOsmConsumerAppBackend::regions()
{
    setBusy(true);
    modules().pramadani_osm_distribution.discoverAsync([this](QString a) { done(a); }, Timeout(120000));
}

void PramadaniOsmConsumerAppBackend::resolve(QString region)
{
    setBusy(true);
    modules().pramadani_osm_distribution.resolveAsync(region, [this](QString a) { done(a); }, Timeout(120000));
}

void PramadaniOsmConsumerAppBackend::download(QString region)
{
    setBusy(true);
    // Country files can be several GB.
    modules().pramadani_osm_distribution.fetchAsync(region, [this](QString a) { done(a); },
                                                    Timeout(6 * 60 * 60 * 1000));
}

void PramadaniOsmConsumerAppBackend::done(const QString& answer)
{
    setAnswer(answer.isEmpty()
                  ? QStringLiteral("{\"ok\":false,\"code\":\"T001\",\"error\":\"pramadani_osm_distribution did not answer\"}")
                  : answer);
    setBusy(false);
}
