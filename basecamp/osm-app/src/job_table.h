#pragma once

// The app backend's job bookkeeping, kept free of Logos and QML so it can be tested on its own
// (tests/cpp/job_table_test.cpp runs it under ASan and UBSan).
//
// A job is one call into pramadani_osm_distribution: {"method", "arg", "state", "answer"} with
// state "submitting", then "confirmed" (the module answered {"ok": true, ...}; for writes that
// means the entry was read back on-chain) or "failed" (an error answer, or no usable answer).

#include <QJsonDocument>
#include <QJsonObject>
#include <QString>
#include <QStringList>

class JobTable
{
public:
    // Finished jobs kept for the Activity list; older ones are dropped (H39: bounded memory).
    static constexpr int kMaxFinished = 200;

    static QString failure(const QString& code, const QString& message)
    {
        QJsonObject o{{"ok", false}, {"code", code}, {"error", message}};
        return QString::fromUtf8(QJsonDocument(o).toJson(QJsonDocument::Compact));
    }

    // Starts a job and returns its id.
    QString start(const QString& method, const QString& arg)
    {
        const QString id = QStringLiteral("%1-%2").arg(method).arg(++m_next);
        m_jobs.insert(id, QJsonObject{{"method", method}, {"arg", arg}, {"state", "submitting"}});
        m_order.append(id);
        return id;
    }

    // Records the module's answer. Unknown ids are ignored; an empty or unparsable answer means
    // the module did not reply (not loaded, or timed out) and becomes an A001 failure.
    void finish(const QString& id, const QString& answer)
    {
        QJsonObject job = m_jobs.value(id).toObject();
        if (job.isEmpty() || job.value("state").toString() != "submitting")
            return;
        QJsonParseError perr;
        const QJsonDocument doc = QJsonDocument::fromJson(answer.toUtf8(), &perr);
        QString text = answer;
        if (perr.error != QJsonParseError::NoError || !doc.isObject())
            text = failure("A001", QStringLiteral("ModuleUnavailable: no answer from pramadani_osm_distribution (not loaded, or timed out)"));
        const bool ok = QJsonDocument::fromJson(text.toUtf8()).object().value("ok").toBool();
        job.insert("state", ok ? "confirmed" : "failed");
        job.insert("answer", text);
        m_jobs.insert(id, job);
        trim();
    }

    // Forgets every finished job; running ones stay.
    void clearFinished()
    {
        QStringList keep;
        for (const QString& id : m_order) {
            if (state(id) == "submitting")
                keep.append(id);
            else
                m_jobs.remove(id);
        }
        m_order = keep;
    }

    QString state(const QString& id) const { return m_jobs.value(id).toObject().value("state").toString(); }
    int size() const { return m_order.size(); }

    QString json() const { return QString::fromUtf8(QJsonDocument(m_jobs).toJson(QJsonDocument::Compact)); }

private:
    void trim()
    {
        int finished = 0;
        for (const QString& id : m_order)
            if (state(id) != "submitting")
                finished++;
        for (int i = 0; i < m_order.size() && finished > kMaxFinished;) {
            if (state(m_order[i]) != "submitting") {
                m_jobs.remove(m_order[i]);
                m_order.removeAt(i);
                finished--;
            } else {
                i++;
            }
        }
    }

    QJsonObject m_jobs;
    QStringList m_order;   // ids, oldest first
    quint64 m_next = 0;
};
