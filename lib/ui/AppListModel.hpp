#pragma once

#include <QAbstractListModel>
#include <QQmlEngine>

#undef signals
#include "../dmenu.hpp"
#define signals public

class AppListModel : public QAbstractListModel
{
    Q_OBJECT

public:
    enum AppRoles {
        NameRole = Qt::UserRole + 1,
        ExecRole,
        IdRole
    };

    explicit AppListModel(QObject *parent = nullptr);

    int rowCount(const QModelIndex &parent = QModelIndex()) const override;
    QVariant data(const QModelIndex &index, int role = Qt::DisplayRole) const override;
    QHash<int, QByteArray> roleNames() const override;

    Q_INVOKABLE void loadApps();

private:
    dmenu::DEVec m_apps;
};