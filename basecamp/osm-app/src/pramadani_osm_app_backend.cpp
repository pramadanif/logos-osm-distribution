#include "pramadani_osm_app_backend.h"

#include <QJsonDocument>

#include "logos_sdk.h"

namespace {

// Downloads and uploads of country files can take hours on a slow link.
constexpr int kLongMs = 6 * 60 * 60 * 1000;
// A write waits up to 120 s for the read-back inside the module; leave room for the fee check.
constexpr int kWriteMs = 5 * 60 * 1000;
constexpr int kReadMs = 2 * 60 * 1000;

QString failure(const QString& code, const QString& message)
{
    QJsonObject o{{"ok", false}, {"code", code}, {"error", message}};
    return QString::fromUtf8(QJsonDocument(o).toJson(QJsonDocument::Compact));
}

} // namespace

QString PramadaniOsmAppBackend::call(QString method, QString arg)
{
    const QString id = QStringLiteral("%1-%2").arg(method).arg(++m_next);
    m_jobs.insert(id, QJsonObject{{"method", method}, {"arg", arg}, {"state", "submitting"}});
    publish();

    auto done = [this, id](QString answer) { finish(id, answer); };
    auto& osm = modules().pramadani_osm_distribution;
    if (method == "version") osm.versionAsync(done, Timeout(kReadMs));
    else if (method == "settings") osm.settingsAsync(done, Timeout(kReadMs));
    else if (method == "configure") osm.configureAsync(arg, done, Timeout(kReadMs));
    else if (method == "discover") osm.discoverAsync(done, Timeout(kReadMs));
    else if (method == "resolve") osm.resolveAsync(arg, done, Timeout(kReadMs));
    else if (method == "hosted_status") osm.hosted_statusAsync(done, Timeout(kReadMs));
    else if (method == "check_updates") osm.check_updatesAsync(arg, done, Timeout(kReadMs));
    else if (method == "fetch") osm.fetchAsync(arg, done, Timeout(kLongMs));
    else if (method == "host") osm.hostAsync(arg, done, Timeout(kLongMs));
    else if (method == "import_file") osm.import_fileAsync(arg, done, Timeout(kLongMs));
    else if (method == "register_entry") osm.register_entryAsync(arg, done, Timeout(kWriteMs));
    else if (method == "unhost") osm.unhostAsync(arg, done, Timeout(kWriteMs));
    else finish(id, failure("A002", QStringLiteral("UnknownAction: %1").arg(method)));
    return id;
}

void PramadaniOsmAppBackend::finish(const QString& id, const QString& answer)
{
    QJsonObject job = m_jobs.value(id).toObject();
    if (job.isEmpty())
        return;
    // An empty or unparsable answer means the module did not reply (not loaded, timed out).
    QJsonParseError perr;
    const QJsonDocument doc = QJsonDocument::fromJson(answer.toUtf8(), &perr);
    QString text = answer;
    if (perr.error != QJsonParseError::NoError || !doc.isObject())
        text = failure("A001", QStringLiteral("ModuleUnavailable: no answer from pramadani_osm_distribution (not loaded, or timed out)"));
    const bool ok = QJsonDocument::fromJson(text.toUtf8()).object().value("ok").toBool();
    job.insert("state", ok ? "confirmed" : "failed");
    job.insert("answer", text);
    m_jobs.insert(id, job);
    publish();
}

void PramadaniOsmAppBackend::clearFinished()
{
    for (const QString& id : m_jobs.keys()) {
        if (m_jobs.value(id).toObject().value("state").toString() != "submitting")
            m_jobs.remove(id);
    }
    publish();
}

void PramadaniOsmAppBackend::publish()
{
    setJobs(QString::fromUtf8(QJsonDocument(m_jobs).toJson(QJsonDocument::Compact)));
}
