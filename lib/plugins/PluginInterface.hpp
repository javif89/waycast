#pragma once

#include "../ui/ListItem.hpp"
#include <vector>
#include <memory>
#include <QString>

namespace plugins
{
    class SearchPlugin
    {
    public:
        virtual ~SearchPlugin() = default;
        
        // Search for items based on query
        virtual std::vector<ListItemPtr> search(const QString &query) = 0;
        
        // Get all items (used for initial display or empty query)
        virtual std::vector<ListItemPtr> getAllItems() = 0;
        
        // Plugin identification
        virtual QString pluginName() const = 0;
        virtual QString pluginDescription() const = 0;
        
        // Priority for ordering results (higher = higher priority)
        virtual int priority() const { return 0; }
        
        // Whether this plugin should be active by default
        virtual bool isEnabled() const { return true; }
    };

    using SearchPluginPtr = std::unique_ptr<SearchPlugin>;
}