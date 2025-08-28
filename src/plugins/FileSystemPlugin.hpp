#pragma once

#include "../../lib/plugins/PluginInterface.hpp"
#include "../../lib/files.hpp"
#include "../../lib/ui/GenericListItem.hpp"
#include <QUrl>
#include <QProcess>
#include <QDir>
#include <filesystem>

namespace plugins
{

    class FileSystemPlugin : public SearchPlugin
    {
    public:
        FileSystemPlugin(const QString& searchPath = QDir::homePath())
            : m_searchPath(searchPath.toStdString())
        {
        }

        std::vector<ListItemPtr> search(const QString& query) override
        {
            std::vector<ListItemPtr> results;
            
            if (query.length() < 2) // Avoid expensive searches for short queries
                return results;

            try {
                for (const auto& entry : std::filesystem::recursive_directory_iterator(m_searchPath)) {
                    if (entry.is_regular_file()) {
                        auto filename = QString::fromStdString(entry.path().filename().string());
                        if (filename.contains(query, Qt::CaseInsensitive)) {
                            auto item = ListItems::createFile(
                                filename,
                                QString::fromStdString(entry.path().string())
                            );
                            results.push_back(item);
                            
                            if (results.size() >= 50) // Limit results to avoid performance issues
                                break;
                        }
                    }
                }
            } catch (const std::filesystem::filesystem_error&) {
                // Handle permission errors silently
            }

            return results;
        }

        std::vector<ListItemPtr> getAllItems() override
        {
            // For file system, don't return all files (too many)
            // Instead return an empty list - files only show up on search
            return {};
        }

        QString pluginName() const override
        {
            return "File System";
        }

        QString pluginDescription() const override
        {
            return "Searches files in the home directory";
        }

        int priority() const override
        {
            return 50; // Lower priority than applications
        }

    private:
        std::string m_searchPath;
    };
}