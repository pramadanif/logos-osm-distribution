// SC-E.7: the app backend's job bookkeeping (basecamp/osm-app/src/job_table.h) under ASan and
// UBSan. Plain asserts; exit 0 means every check passed. Build and run: tests/cpp/run.sh.

#include "job_table.h"

#include <QJsonArray>
#include <cstdio>
#include <cstdlib>

static int failures = 0;
#define CHECK(cond)                                                              \
    do {                                                                         \
        if (!(cond)) {                                                           \
            std::fprintf(stderr, "FAIL %s:%d %s\n", __FILE__, __LINE__, #cond);  \
            failures++;                                                          \
        }                                                                        \
    } while (0)

static QJsonObject jobs(const JobTable& t)
{
    return QJsonDocument::fromJson(t.json().toUtf8()).object();
}

int main()
{
    JobTable t;

    // A job starts as submitting and gets a distinct id.
    const QString a = t.start("resolve", "china/henan");
    const QString b = t.start("resolve", "kenya");
    CHECK(a != b);
    CHECK(t.state(a) == "submitting");
    CHECK(jobs(t).value(a).toObject().value("arg").toString() == "china/henan");

    // ok:true confirms; ok:false fails; the answer is kept as the module sent it.
    t.finish(a, R"({"ok":true,"data":{"cid":"zDvZRwzmA5Lr"}})");
    CHECK(t.state(a) == "confirmed");
    t.finish(b, R"({"ok":false,"code":"S160","error":"bad"})");
    CHECK(t.state(b) == "failed");
    CHECK(jobs(t).value(b).toObject().value("answer").toString().contains("S160"));

    // A job finishes once; a late second answer does not flip it.
    t.finish(a, R"({"ok":false,"code":"S103","error":"late"})");
    CHECK(t.state(a) == "confirmed");

    // No usable answer (empty, garbage, not an object) is an A001 failure, never a success.
    for (const char* bad : {"", "not json", "[1,2,3]", "{", "\"ok\""}) {
        const QString id = t.start("fetch", "kenya");
        t.finish(id, QString::fromUtf8(bad));
        CHECK(t.state(id) == "failed");
        CHECK(jobs(t).value(id).toObject().value("answer").toString().contains("A001"));
    }

    // An unknown id is ignored.
    const int before = t.size();
    t.finish("nope-999", R"({"ok":true})");
    CHECK(t.size() == before);

    // Markup and very long text are stored as given (the view renders plain text).
    const QString big(1 << 20, QChar('x'));
    const QString c = t.start("lookup", "<b>" + big + "</b>");
    t.finish(c, QStringLiteral(R"({"ok":true,"data":"%1"})").arg(big));
    CHECK(t.state(c) == "confirmed");

    // clearFinished keeps running jobs only.
    const QString running = t.start("host", R"({"region":"kenya"})");
    t.clearFinished();
    CHECK(t.size() == 1);
    CHECK(t.state(running) == "submitting");

    // At most kMaxFinished finished jobs are kept, oldest dropped first; running ones stay.
    QString first;
    for (int i = 0; i < JobTable::kMaxFinished + 50; i++) {
        const QString id = t.start("resolve", QString::number(i));
        if (i == 0)
            first = id;
        t.finish(id, R"({"ok":true,"data":null})");
    }
    CHECK(t.size() == JobTable::kMaxFinished + 1);
    CHECK(t.state(first).isEmpty());
    CHECK(t.state(running) == "submitting");

    // The failure helper builds a parsable envelope.
    const QJsonObject f = QJsonDocument::fromJson(JobTable::failure("A002", "UnknownAction: \"x\"").toUtf8()).object();
    CHECK(f.value("ok").toBool() == false && f.value("code").toString() == "A002");

    std::printf("%s (%d failures)\n", failures ? "FAIL" : "PASS", failures);
    return failures ? 1 : 0;
}
