#pragma once

#include <QString>
#include <QUrl>
#include <memory>

class ListItem
{
public:
    virtual ~ListItem() = default;

    // Display properties for the UI
    virtual QString name() const = 0;
    virtual QString description() const = 0;  // Optional subtitle/description
    virtual QUrl iconUrl() const = 0;
    
    // Action when item is selected
    virtual void execute() = 0;
    
    // Search matching - return true if this item matches the search query
    virtual bool matches(const QString &query) const;
    
    // Item type for extensibility (e.g., "app", "file", "bookmark")
    virtual QString itemType() const = 0;

protected:
    // Helper for default search matching (case-insensitive name search)
    bool defaultMatches(const QString &query) const;
};

using ListItemPtr = std::shared_ptr<ListItem>;