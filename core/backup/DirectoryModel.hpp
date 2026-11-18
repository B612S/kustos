// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#pragma once

#include <QAbstractListModel>
#include <QDateTime>
#include <QString>

class SnapshotStore;

// Read-only listing of one directory inside a snapshot. The app browses a
// snapshot like a file manager: enter a folder, go up, select entries. Nothing
// here touches the live home directory.
class DirectoryModel : public QAbstractListModel {
    Q_OBJECT
    Q_PROPERTY(QString snapshotId READ snapshotId WRITE setSnapshotId NOTIFY pathChanged)
    Q_PROPERTY(QString relativePath READ relativePath WRITE setRelativePath NOTIFY pathChanged)
    Q_PROPERTY(QString displayPath READ displayPath NOTIFY pathChanged)
    Q_PROPERTY(bool canGoUp READ canGoUp NOTIFY pathChanged)
    Q_PROPERTY(int count READ rowCount NOTIFY countChanged)

public:
    enum Roles {
        NameRole = Qt::UserRole + 1,
        RelPathRole,
        IsDirRole,
        IsLinkRole,
        SizeRole,
        SizeTextRole,
        ModifiedRole,
        ModifiedTextRole,
    };
    Q_ENUM(Roles)

    explicit DirectoryModel(SnapshotStore *store, QObject *parent = nullptr);

    int rowCount(const QModelIndex &parent = QModelIndex()) const override;
    QVariant data(const QModelIndex &index, int role) const override;
    QHash<int, QByteArray> roleNames() const override;

    QString snapshotId() const { return m_snapshotId; }
    Q_INVOKABLE void setSnapshotId(const QString &id);
    QString relativePath() const { return m_relativePath; }
    void setRelativePath(const QString &path);
    QString displayPath() const;
    bool canGoUp() const;

    Q_INVOKABLE bool enter(int row);
    Q_INVOKABLE void goUp();
    Q_INVOKABLE QVariantMap get(int row) const;
    Q_INVOKABLE QString absolutePathFor(const QString &relative) const;

signals:
    void pathChanged();
    void countChanged();

private:
    struct Entry {
        QString name;
        QString relative;
        qint64 size = 0;
        QDateTime modified;
        bool isDir = false;
        bool isLink = false;
    };

    void reload();
    QString absolutePath() const;

    SnapshotStore *m_store = nullptr;
    QString m_snapshotId;
    QString m_relativePath;
    QList<Entry> m_entries;
};
