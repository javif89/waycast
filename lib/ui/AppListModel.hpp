#pragma once

#include <QAbstractListModel>
#include <QQmlEngine>

#undef signals
#include "../dmenu.hpp"
#define signals public

class AppListModel : public QAbstractListModel
{
    Q_OBJECT
    Q_PROPERTY(QString searchText READ searchText WRITE setSearchText NOTIFY searchTextChanged)

public:
    enum AppRoles {
        NameRole = Qt::UserRole + 1,
        ExecRole,
        IdRole,
        IconRole
    };

    explicit AppListModel(QObject *parent = nullptr);

    int rowCount(const QModelIndex &parent = QModelIndex()) const override;
    QVariant data(const QModelIndex &index, int role = Qt::DisplayRole) const override;
    QHash<int, QByteArray> roleNames() const override;

    Q_INVOKABLE void loadApps();
    Q_INVOKABLE void launchApp(int index);

    QString searchText() const;
    void setSearchText(const QString &searchText);

signals:
    void searchTextChanged();

private:
    void updateFilteredApps();
    QUrl getIconUrl(const dmenu::DesktopEntry &app) const;

    dmenu::DEVec m_apps;
    std::vector<int> m_filteredIndexes;
    QString m_searchText;
};