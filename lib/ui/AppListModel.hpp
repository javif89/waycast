#pragma once

#include <QAbstractListModel>
#include <QQmlEngine>
#include "ListItem.hpp"
#include <vector>

class AppListModel : public QAbstractListModel
{
    Q_OBJECT
    Q_PROPERTY(QString searchText READ searchText WRITE setSearchText NOTIFY searchTextChanged)

public:
    enum ItemRoles
    {
        NameRole = Qt::UserRole + 1,
        DescriptionRole,
        IconRole,
        ItemTypeRole
    };

    explicit AppListModel(QObject *parent = nullptr);

    int rowCount(const QModelIndex &parent = QModelIndex()) const override;
    QVariant data(const QModelIndex &index, int role = Qt::DisplayRole) const override;
    QHash<int, QByteArray> roleNames() const override;

    Q_INVOKABLE void loadItems();
    Q_INVOKABLE void executeItem(int index);

    // Add items from different sources
    void addItems(const std::vector<ListItemPtr> &items);

    QString searchText() const;
    void setSearchText(const QString &searchText);

signals:
    void searchTextChanged();

private:
    void updateFilteredItems();

    std::vector<ListItemPtr> m_items;
    std::vector<int> m_filteredIndexes;
    QString m_searchText;
};