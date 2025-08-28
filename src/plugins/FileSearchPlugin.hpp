#pragma once

#include "PluginInterface.hpp"
#include "GenericListItem.hpp"
#include "IconUtil.hpp"
#include "files.hpp"
#include "fuzzy.hpp"
#include <QUrl>
#include <QProcess>
#include <QDir>
#include <QStandardPaths>
#include <filesystem>
#include <unordered_set>
#include <algorithm>

namespace plugins
{
    class FileSearchPlugin : public SearchPlugin
    {
    public:
        // Constructor with configurable search and ignore directories
        explicit FileSearchPlugin(
            const std::vector<std::string> &searchDirectories = getDefaultSearchDirs(),
            int maxDepth = 3,
            size_t maxFiles = 1000)
            : m_searchDirectories(searchDirectories), m_maxDepth(maxDepth), m_maxFiles(maxFiles)
        {
            // Initialize ignore directory names set
            m_ignoreDirNames = {
                "node_modules", "vendor", ".git", ".svn", ".hg", 
                "build", "dist", "target", ".cache", "__pycache__",
                ".pytest_cache", ".mypy_cache", "coverage", ".coverage",
                ".tox", "venv", ".venv", "env", ".env"
            };

            // Pre-load all files from search directories
            loadFiles();
        }

        std::vector<ListItemPtr> search(const QString &query) override
        {
            std::vector<ListItemPtr> results;

            if (query.length() < 2)
            {
                return results; // Require at least 2 characters to avoid huge result sets
            }

            // Check if query looks like a path (contains / or ends with /)
            bool isPathQuery = query.contains('/');
            std::string pathFilter;
            std::string searchTerm = query.toStdString();
            
            if (isPathQuery)
            {
                // Extract path component and search term
                QString queryStr = query;
                int lastSlash = queryStr.lastIndexOf('/');
                
                if (lastSlash != -1)
                {
                    pathFilter = queryStr.left(lastSlash + 1).toStdString(); // Include the trailing slash
                    searchTerm = queryStr.mid(lastSlash + 1).toStdString();  // Filename part after last slash
                    
                    // If search term is empty (query ends with /), search for any file in that path
                    if (searchTerm.empty())
                    {
                        searchTerm = ""; // Will match all files in the path
                    }
                }
                else
                {
                    pathFilter = queryStr.toStdString();
                    searchTerm = "";
                }
            }

            // Filter files based on path if this is a path query
            std::vector<std::filesystem::path> candidateFiles;
            if (isPathQuery && !pathFilter.empty())
            {
                for (const auto &file : m_allFiles)
                {
                    std::string fullPath = file.string();
                    // Check if the file path contains the path filter
                    if (fullPath.find(pathFilter) != std::string::npos)
                    {
                        candidateFiles.push_back(file);
                    }
                }
            }
            else
            {
                candidateFiles = m_allFiles; // Use all files for regular search
            }

            // Convert file paths to strings for fuzzy matching
            std::vector<std::string> filePaths;
            filePaths.reserve(candidateFiles.size());

            for (const auto &file : candidateFiles)
            {
                filePaths.push_back(file.filename().string()); // Match against filename only
            }

            // Use fuzzy finder to get matches
            std::vector<fuzzy::FuzzyMatch> fuzzyMatches;
            if (searchTerm.empty())
            {
                // If no search term (just path/), return all files in that path
                for (size_t i = 0; i < filePaths.size() && i < 50; ++i)
                {
                    fuzzy::FuzzyMatch match;
                    match.text = filePaths[i];
                    match.score = 100; // High score for direct path matches
                    fuzzyMatches.push_back(match);
                }
            }
            else
            {
                fuzzyMatches = m_fuzzyFinder.find(filePaths, searchTerm, 50);
            }

            // Convert fuzzy matches back to ListItems
            results.reserve(fuzzyMatches.size());
            for (const auto &match : fuzzyMatches)
            {
                // Find the original file path that corresponds to this match
                auto it = std::find_if(candidateFiles.begin(), candidateFiles.end(),
                                       [&match](const std::filesystem::path &path)
                                       {
                                           return path.filename().string() == match.text;
                                       });

                if (it != candidateFiles.end())
                {
                    auto item = createFileListItem(*it, match.score);
                    results.push_back(item);
                }
            }

            return results;
        }

        std::vector<ListItemPtr> getAllItems() override
        {
            // Don't return all files by default (too many)
            // Could return recent files or empty list
            return {};
        }

        QString pluginName() const override
        {
            return "File Search";
        }

        QString pluginDescription() const override
        {
            return QString("Searches files in specified directories (currently monitoring %1 files)")
                .arg(m_allFiles.size());
        }

        int priority() const override
        {
            return 25; // Lower priority than applications, higher than examples
        }

        // Configuration methods
        void addSearchDirectory(const std::string &directory)
        {
            m_searchDirectories.push_back(directory);
            loadFiles(); // Reload files
        }

        void addIgnoreDirName(const std::string &dirName)
        {
            m_ignoreDirNames.insert(dirName);
            loadFiles(); // Reload files
        }

        size_t getFileCount() const { return m_allFiles.size(); }

    private:
        // Default search directories
        static std::vector<std::string> getDefaultSearchDirs()
        {
            std::vector<std::string> dirs;

            // Add common user directories
            QString home = QDir::homePath();
            dirs.push_back((home + "/Documents").toStdString());
            dirs.push_back((home + "/Desktop").toStdString());
            dirs.push_back((home + "/Downloads").toStdString());

            // Add XDG user directories if they exist
            auto xdgDirs = QStandardPaths::standardLocations(QStandardPaths::DocumentsLocation);
            for (const QString &dir : xdgDirs)
            {
                if (QDir(dir).exists())
                {
                    dirs.push_back(dir.toStdString());
                }
            }

            return dirs;
        }


        void loadFiles()
        {
            m_allFiles.clear();

            for (const auto &searchDir : m_searchDirectories)
            {
                if (!std::filesystem::exists(searchDir))
                {
                    continue;
                }

                try
                {
                    // Get all files from this directory
                    auto files = files::findAllFiles(searchDir, m_maxDepth);

                    for (const auto &file : files)
                    {
                        // Check if this file should be ignored
                        if (shouldIgnoreFile(file))
                        {
                            continue;
                        }

                        m_allFiles.push_back(file);

                        // Limit total files to prevent memory issues
                        if (m_allFiles.size() >= m_maxFiles)
                        {
                            return;
                        }
                    }
                }
                catch (const std::filesystem::filesystem_error &e)
                {
                    // Silently skip directories we can't access
                    continue;
                }
            }
        }

        bool shouldIgnoreFile(const std::filesystem::path &file) const
        {
            try
            {
                // Check if file is in any ignored directory (walk up the path)
                auto current = file.parent_path();
                while (!current.empty() && current != current.root_path())
                {
                    std::string dirName = current.filename().string();
                    
                    // Ignore any directory that starts with .
                    if (!dirName.empty() && dirName[0] == '.')
                    {
                        return true;
                    }
                    
                    // Ignore directories by name
                    if (m_ignoreDirNames.contains(dirName))
                    {
                        return true;
                    }
                    
                    current = current.parent_path();
                }

                // Ignore hidden files (starting with .)
                std::string fileName = file.filename().string();
                if (!fileName.empty() && fileName[0] == '.')
                {
                    return true;
                }

                // Ignore common temporary file extensions
                auto extension = file.extension().string();
                std::transform(extension.begin(), extension.end(), extension.begin(), ::tolower);
                if (extension == ".tmp" || extension == ".temp" || 
                    extension == ".log" || extension == ".cache")
                {
                    return true;
                }

                return false;
            }
            catch (const std::filesystem::filesystem_error &)
            {
                return true; // Ignore files we can't access
            }
        }

        ListItemPtr createFileListItem(const std::filesystem::path &filePath, int fuzzyScore)
        {
            QString fileName = QString::fromStdString(filePath.filename().string());
            QString fullPath = QString::fromStdString(filePath.string());
            QString parentDir = QString::fromStdString(filePath.parent_path().filename().string());

            // Create description showing parent directory
            QString description = parentDir.isEmpty() ? fullPath : QString("%1/.../%2").arg(parentDir, fileName);

            // Get icon using shared utility
            QUrl icon = IconUtil::getFileIcon(filePath);

            return ListItems::createFile(fileName, fullPath, icon);
        }


        // Member variables
        std::vector<std::string> m_searchDirectories;
        std::unordered_set<std::string> m_ignoreDirNames;
        int m_maxDepth;
        size_t m_maxFiles;

        std::vector<std::filesystem::path> m_allFiles;
        fuzzy::FuzzyFinder m_fuzzyFinder;
    };
}