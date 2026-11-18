// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

#include "backup/DirectoryModel.hpp"

#include "backup/SnapshotStore.hpp"
#include "util/Format.hpp"

#include <QDir>
#include <QFileInfo>
#include <QJsonObject>

#include <algorithm>

DirectoryModel::DirectoryModel(SnapshotStore *store, QObject *parent)
    : QAbstractListModel(parent)
    , m_store(store)
{
}

int DirectoryModel::rowCount(const QModelIndex &parent) const
{
    return parent.isValid() ? 0 : static_cast<int>(m_entries.size());
}

QVariant DirectoryModel::data(const QModelIndex &index, int role) const
{
    if (!index.isValid() || index.row() < 0 || index.row() >= m_entries.size())
        return {};

    const Entry &entry = m_entries.at(index.row());
    switch (role) {
    case NameRole:
        return entry.name;
    case RelPathRole:
        return entry.relative;
    case IsDirRole:
        return entry.isDir;
    case IsLinkRole:
        return entry.isLink;
    case SizeRole:
        return QVariant::fromValue(entry.size);
    case SizeTextRole:
        return entry.isDir ? QString() : Format::bytes(entry.size);
    case ModifiedRole:
        return entry.modified;
    case ModifiedTextRole:
        return entry.modified.isValid()
            ? QLocale::system().toString(entry.modified, QLocale::ShortFormat)
            : QString();
    default:
        return {};
    }
}

QHash<int, QByteArray> DirectoryModel::roleNames() const
{
    return {
        {NameRole, QByteArrayLiteral("name")},
        {RelPathRole, QByteArrayLiteral("relativePath")},
        {IsDirRole, QByteArrayLiteral("isDir")},
        {IsLinkRole, QByteArrayLiteral("isLink")},
        {SizeRole, QByteArrayLiteral("size")},
        {SizeTextRole, QByteArrayLiteral("sizeText")},
        {ModifiedRole, QByteArrayLiteral("modified")},
        {ModifiedTextRole, QByteArrayLiteral("modifiedText")},
    };
}

void DirectoryModel::setSnapshotId(const QString &id)
{
    if (m_snapshotId == id)
        return;
    m_snapshotId = id;
    // Always start at the user's home directory inside the snapshot; fall back
    // to the snapshot root when the backup was made for another user name.
    m_relativePath = m_store->homeRoot();
    if (!QFileInfo::exists(absolutePath()))
        m_relativePath.clear();
    reload();
    emit pathChanged();
}

void DirectoryModel::setRelativePath(const QString &path)
{
    QString cleaned = QDir::cleanPath(path);
    if (cleaned == QLatin1String("."))
        cleaned.clear();
    if (cleaned.startsWith(QLatin1Char('/')))
        cleaned.remove(0, 1);
    if (cleaned.contains(QLatin1String("..")))
        return; // never let the browser escape the snapshot
    if (m_relativePath == cleaned)
        return;
    m_relativePath = cleaned;
    if (!QFileInfo::exists(absolutePath()))
        m_relativePath.clear();
    reload();
    emit pathChanged();
}

QString DirectoryModel::displayPath() const
{
    if (m_relativePath.isEmpty())
        return QStringLiteral("/");
    const QString root = m_store->homeRoot();
    if (m_relativePath == root)
        return QStringLiteral("~");
    if (m_relativePath.startsWith(root + QLatin1Char('/')))
        return QStringLiteral("~") + m_relativePath.mid(root.size());
    return m_relativePath;
}

bool DirectoryModel::canGoUp() const
{
    return !m_relativePath.isEmpty();
}

QString DirectoryModel::absolutePath() const
{
    QString path = m_store->snapshotPath(m_snapshotId);
    if (!m_relativePath.isEmpty())
        path += QLatin1Char('/') + m_relativePath;
    return path;
}

QString DirectoryModel::absolutePathFor(const QString &relative) const
{
    QString path = m_store->snapshotPath(m_snapshotId);
    if (!relative.isEmpty())
        path += QLatin1Char('/') + relative;
    return path;
}

bool DirectoryModel::enter(int row)
{
    if (row < 0 || row >= m_entries.size())
        return false;
    const Entry &entry = m_entries.at(row);
    if (!entry.isDir)
        return false;
    setRelativePath(entry.relative);
    return true;
}

void DirectoryModel::goUp()
{
    if (m_relativePath.isEmpty())
        return;
    const int slash = m_relativePath.lastIndexOf(QLatin1Char('/'));
    setRelativePath(slash < 0 ? QString() : m_relativePath.left(slash));
}

QVariantMap DirectoryModel::get(int row) const
{
    QVariantMap map;
    if (row < 0 || row >= m_entries.size())
        return map;
    const Entry &entry = m_entries.at(row);
    map.insert(QStringLiteral("name"), entry.name);
    map.insert(QStringLiteral("relativePath"), entry.relative);
    map.insert(QStringLiteral("isDir"), entry.isDir);
    map.insert(QStringLiteral("size"), QVariant::fromValue(entry.size));
    map.insert(QStringLiteral("sizeText"), entry.isDir ? QString() : Format::bytes(entry.size));
    map.insert(QStringLiteral("modified"), entry.modified);
    map.insert(QStringLiteral("absolutePath"), absolutePathFor(entry.relative));
    return map;
}

void DirectoryModel::reload()
{
    beginResetModel();
    m_entries.clear();

    const QString path = absolutePath();
    const QDir directory(path);
    if (!path.isEmpty() && directory.exists()) {
        const QFileInfoList entries = directory.entryInfoList(
            QDir::AllEntries | QDir::NoDotAndDotDot | QDir::Hidden | QDir::System, QDir::NoSort);
        m_entries.reserve(entries.size());
        for (const QFileInfo &info : entries) {
            // The snapshot's own metadata is not user data.
            if (m_relativePath.isEmpty() && info.fileName() == QLatin1String("meta.json"))
                continue;
            Entry entry;
            entry.name = info.fileName();
            entry.relative = m_relativePath.isEmpty() ? entry.name
                                                      : m_relativePath + QLatin1Char('/') + entry.name;
            entry.isLink = info.isSymLink();
            // isDir() follows symlinks, which is what a file browser should do.
            entry.isDir = info.isDir();
            entry.size = entry.isDir ? 0 : info.size();
            entry.modified = info.lastModified();
            m_entries.append(entry);
        }
    }

    std::sort(m_entries.begin(), m_entries.end(), [](const Entry &a, const Entry &b) {
        if (a.isDir != b.isDir)
            return a.isDir;
        return a.name.compare(b.name, Qt::CaseInsensitive) < 0;
    });

    endResetModel();
    emit countChanged();
}
