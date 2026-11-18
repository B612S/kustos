// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#pragma once

#include <QDateTime>
#include <QJsonObject>
#include <QString>
#include <QStringList>

// One point on the timeline: the contents of meta.json inside a snapshot
// directory, plus the parsed values used by the UI and by the retention
// policy.
struct SnapshotInfo {
    QString id;                       // "2026-11-18_19-00-00"
    QDateTime startedAt;
    QDateTime finishedAt;
    QString status = QStringLiteral("ok"); // ok | warning | failed | running
    qint64 totalBytes = 0;
    qint64 totalFiles = 0;
    qint64 transferredBytes = 0;
    qint64 transferredFiles = 0;
    qint64 durationSeconds = 0;
    int rsyncExitCode = 0;
    QString error;
    QStringList warnings;
    QString hostname;
    QString appVersion;
    QStringList sources;
    QStringList excludes;

    bool isValid() const { return !id.isEmpty(); }
};

QJsonObject snapshotToJson(const SnapshotInfo &info);
SnapshotInfo snapshotFromJson(const QJsonObject &object);
