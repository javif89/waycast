#include "AppListModel.hpp"
#include "DesktopAppListItem.hpp"
#include <QDebug>

#undef signals
#include "../dmenu.hpp"
#define signals public

AppListModel::AppListModel(QObject *parent)
    : QAbstractListModel(parent)
{
    loadItems();
}

int AppListModel::rowCount(const QModelIndex &parent) const
{
    Q_UNUSED(parent);
    return static_cast<int>(m_filteredIndexes.size());
}

QVariant AppListModel::data(const QModelIndex &index, int role) const
{
    if (!index.isValid() || index.row() >= static_cast<int>(m_filteredIndexes.size()))
        return QVariant();

    int itemIndex = m_filteredIndexes[index.row()];
    if (itemIndex >= static_cast<int>(m_items.size()))
        return QVariant();

    const ListItemPtr &item = m_items[itemIndex];

    switch (role) {
    case NameRole:
        return item->name();
    case DescriptionRole:
        return item->description();
    case IconRole:
        return item->iconUrl();
    case ItemTypeRole:
        return item->itemType();
    default:
        return QVariant();
    }
}

QHash<int, QByteArray> AppListModel::roleNames() const
{
    QHash<int, QByteArray> roles;
    roles[NameRole] = "name";
    roles[DescriptionRole] = "description";
    roles[IconRole] = "icon";
    roles[ItemTypeRole] = "itemType";
    return roles;
}

void AppListModel::loadItems()
{
    beginResetModel();
    m_items.clear();
    
    // Add desktop applications
    addDesktopApps();
    
    updateFilteredItems();
    endResetModel();
}

void AppListModel::executeItem(int index)
{
    if (index < 0 || index >= static_cast<int>(m_filteredIndexes.size()))
        return;
        
    int itemIndex = m_filteredIndexes[index];
    if (itemIndex >= static_cast<int>(m_items.size()))
        return;

    m_items[itemIndex]->execute();
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
    updateFilteredItems();
    endResetModel();
}

void AppListModel::addDesktopApps()
{
    dmenu::DEVec apps = dmenu::get_dmenu_app_data();
    if (!apps)
        return;
        
    for (const auto &entry : *apps) {
        auto item = std::make_shared<DesktopAppListItem>(entry);
        m_items.push_back(item);
    }
}

void AppListModel::addItems(const std::vector<ListItemPtr> &items)
{
    for (const auto &item : items) {
        m_items.push_back(item);
    }
}

void AppListModel::updateFilteredItems()
{
    m_filteredIndexes.clear();
    
    for (size_t i = 0; i < m_items.size(); ++i) {
        const ListItemPtr &item = m_items[i];
        
        if (item->matches(m_searchText)) {
            m_filteredIndexes.push_back(static_cast<int>(i));
        }
    }
}