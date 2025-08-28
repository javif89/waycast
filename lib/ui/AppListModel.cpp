#include "AppListModel.hpp"
#include <QDebug>

AppListModel::AppListModel(QObject *parent)
    : QAbstractListModel(parent)
{
    loadApps();
}

int AppListModel::rowCount(const QModelIndex &parent) const
{
    Q_UNUSED(parent);
    return m_apps ? static_cast<int>(m_apps->size()) : 0;
}

QVariant AppListModel::data(const QModelIndex &index, int role) const
{
    if (!index.isValid() || !m_apps || index.row() >= static_cast<int>(m_apps->size()))
        return QVariant();

    const dmenu::DesktopEntry &app = (*m_apps)[index.row()];

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
    endResetModel();
}