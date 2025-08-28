#pragma once

#include "PluginInterface.hpp"
#include "dmenu.hpp"
#include "DesktopAppPlugin/DesktopAppListItem.hpp"
#include <algorithm>

namespace plugins
{
    class DesktopAppPlugin : public SearchPlugin
    {
    public:
        DesktopAppPlugin()
        {
            loadDesktopEntries();
        }

        std::vector<ListItemPtr> search(const QString &query) override
        {
            std::vector<ListItemPtr> results;
            
            for (const auto &item : m_allItems)
            {
                if (item->matches(query))
                {
                    results.push_back(item);
                }
            }
            
            return results;
        }

        std::vector<ListItemPtr> getAllItems() override
        {
            return m_allItems;
        }

        QString pluginName() const override
        {
            return "Desktop Applications";
        }

        QString pluginDescription() const override
        {
            return "Searches installed desktop applications";
        }

        int priority() const override
        {
            return 100; // High priority for applications
        }

    private:
        void loadDesktopEntries()
        {
            auto desktopEntries = dmenu::get_dmenu_app_data();
            
            for (const auto &entry : *desktopEntries)
            {
                auto listItem = std::make_shared<DesktopAppListItem>(entry);
                m_allItems.push_back(listItem);
            }
        }

        std::vector<ListItemPtr> m_allItems;
    };
}