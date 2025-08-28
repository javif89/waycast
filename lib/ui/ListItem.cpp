#include "ListItem.hpp"

bool ListItem::matches(const QString &query) const
{
    return defaultMatches(query);
}

bool ListItem::defaultMatches(const QString &query) const
{
    if (query.isEmpty())
        return true;
        
    return name().contains(query, Qt::CaseInsensitive) ||
           description().contains(query, Qt::CaseInsensitive);
}