#pragma once

#include "rep_pramadani_osm_consumer_app_source.h"
#include "logos_module_context.h"

// Template backend: forwards to pramadani_osm_distribution and publishes the answer.
class PramadaniOsmConsumerAppBackend : public PramadaniOsmConsumerAppSimpleSource,
                                       public LogosModuleContext
{
public:
    void regions() override;
    void resolve(QString region) override;
    void download(QString region) override;

private:
    void done(const QString& answer);
};
