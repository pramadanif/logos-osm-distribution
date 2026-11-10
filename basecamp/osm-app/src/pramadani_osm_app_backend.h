#pragma once

#include <QJsonObject>

#include "rep_pramadani_osm_app_source.h"
#include "logos_module_context.h"

// Backend of the OSM app. Holds no keys and no map logic: every action is a call into the
// pramadani_osm_distribution core module (which signs through lez_core and confirms writes by
// reading the region account back). The view sees each call as a job in the `jobs` PROP.
class PramadaniOsmAppBackend : public PramadaniOsmAppSimpleSource,
                               public LogosModuleContext
{
public:
    QString call(QString method, QString arg) override;
    void clearFinished() override;

private:
    void finish(const QString& id, const QString& answer);
    void publish();

    QJsonObject m_jobs;
    quint64 m_next = 0;
};
