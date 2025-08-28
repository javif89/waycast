#pragma once

#include "../../lib/plugins/PluginInterface.hpp"
#include "../../lib/ui/GenericListItem.hpp"

namespace plugins
{
    // Example of how simple it is to create a new plugin
    class ExamplePlugin : public SearchPlugin
    {
    public:
        std::vector<ListItemPtr> search(const QString& query) override
        {
            std::vector<ListItemPtr> results;
            
            // Example: Create some sample items that match any query
            if (query.contains("test", Qt::CaseInsensitive)) {
                // Using the convenient factory function
                results.push_back(ListItems::createItem(
                    "Test Item 1", 
                    "This is a test item", 
                    "example",
                    []() { /* custom action */ }
                ));
                
                // Or create directly with GenericListItem
                results.push_back(std::make_shared<GenericListItem>(
                    "Test Item 2",
                    "Another test item",
                    QUrl(), // no icon
                    "example",
                    []() { 
                        // Custom execute action
                        qDebug() << "Test item executed!";
                    }
                ));
            }
            
            return results;
        }

        std::vector<ListItemPtr> getAllItems() override
        {
            // Return some default items
            return {
                ListItems::createItem("Example Item", "Always visible", "example")
            };
        }

        QString pluginName() const override { return "Example Plugin"; }
        QString pluginDescription() const override { return "Demonstrates easy plugin creation"; }
        int priority() const override { return 10; } // Low priority
    };
}