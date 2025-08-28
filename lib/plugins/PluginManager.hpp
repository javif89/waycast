#pragma once

#include "PluginInterface.hpp"
#include "../ui/ListItem.hpp"
#include <vector>
#include <memory>
#include <algorithm>

namespace plugins
{
    class PluginManager
    {
    public:
        static PluginManager& instance()
        {
            static PluginManager instance;
            return instance;
        }

        void registerPlugin(SearchPluginPtr plugin)
        {
            if (plugin && plugin->isEnabled())
            {
                m_plugins.push_back(std::move(plugin));
                
                // Sort plugins by priority (higher priority first)
                std::sort(m_plugins.begin(), m_plugins.end(), 
                    [](const SearchPluginPtr& a, const SearchPluginPtr& b) {
                        return a->priority() > b->priority();
                    });
            }
        }

        std::vector<ListItemPtr> search(const QString& query)
        {
            std::vector<ListItemPtr> allResults;
            
            for (const auto& plugin : m_plugins)
            {
                if (query.isEmpty())
                {
                    auto items = plugin->getAllItems();
                    allResults.insert(allResults.end(), items.begin(), items.end());
                }
                else
                {
                    auto results = plugin->search(query);
                    allResults.insert(allResults.end(), results.begin(), results.end());
                }
            }
            
            return allResults;
        }

        std::vector<ListItemPtr> getAllItems()
        {
            std::vector<ListItemPtr> allItems;
            
            for (const auto& plugin : m_plugins)
            {
                auto items = plugin->getAllItems();
                allItems.insert(allItems.end(), items.begin(), items.end());
            }
            
            return allItems;
        }

        const std::vector<SearchPluginPtr>& getPlugins() const
        {
            return m_plugins;
        }

    private:
        PluginManager() = default;
        std::vector<SearchPluginPtr> m_plugins;
    };
}