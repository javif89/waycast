#include "AppListModel.hpp"
#include <QDebug>
#include <QProcess>
#include <QRegularExpression>

AppListModel::AppListModel(QObject *parent)
    : QAbstractListModel(parent)
{
    loadApps();
}

int AppListModel::rowCount(const QModelIndex &parent) const
{
    Q_UNUSED(parent);
    return static_cast<int>(m_filteredIndexes.size());
}

QVariant AppListModel::data(const QModelIndex &index, int role) const
{
    if (!index.isValid() || !m_apps || index.row() >= static_cast<int>(m_filteredIndexes.size()))
        return QVariant();

    int appIndex = m_filteredIndexes[index.row()];
    const dmenu::DesktopEntry &app = (*m_apps)[appIndex];

    switch (role) {
    case NameRole:
        return QString::fromStdString(app.name);
    case ExecRole:
        return QString::fromStdString(app.exec);
    case IdRole:
        return QString::fromStdString(app.id);
    default:
        return QVariant();
    }
}

QHash<int, QByteArray> AppListModel::roleNames() const
{
    QHash<int, QByteArray> roles;
    roles[NameRole] = "name";
    roles[ExecRole] = "exec";
    roles[IdRole] = "id";
    return roles;
}

void AppListModel::loadApps()
{
    beginResetModel();
    m_apps = dmenu::get_dmenu_app_data();
    updateFilteredApps();
    endResetModel();
}

void AppListModel::launchApp(int index)
{
    if (!m_apps || index < 0 || index >= static_cast<int>(m_filteredIndexes.size()))
        return;
        
    int appIndex = m_filteredIndexes[index];
    const dmenu::DesktopEntry &app = (*m_apps)[appIndex];
    
    // Parse exec command (remove %f, %u, %F, %U field codes if present)
    QString command = QString::fromStdString(app.exec);
    QRegularExpression fieldCodes("%[fuFU]");
    command = command.replace(fieldCodes, "").trimmed();
    
    qDebug() << "Launching:" << command;
    
    // Use nohup and redirect output to /dev/null for proper detachment
    QString detachedCommand = QString("nohup %1 >/dev/null 2>&1 &").arg(command);
    QProcess::startDetached("/bin/sh", QStringList() << "-c" << detachedCommand);
}

QString AppListModel::searchText() const
{
    return m_searchText;
}

void AppListModel::setSearchText(const QString &searchText)
{
    if (m_searchText == searchText)
        return;

    m_searchText = searchText;
    emit searchTextChanged();
    
    beginResetModel();
    updateFilteredApps();
    endResetModel();
}

void AppListModel::updateFilteredApps()
{
    m_filteredIndexes.clear();
    
    if (!m_apps)
        return;
    
    for (size_t i = 0; i < m_apps->size(); ++i) {
        const dmenu::DesktopEntry &app = (*m_apps)[i];
        
        if (m_searchText.isEmpty() || 
            QString::fromStdString(app.name).contains(m_searchText, Qt::CaseInsensitive)) {
            m_filteredIndexes.push_back(static_cast<int>(i));
        }
    }
}