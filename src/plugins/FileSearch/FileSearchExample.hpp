#pragma once

#include "FileSearchPlugin.hpp"
#include <QDir>

namespace plugins
{
    // Example configurations for the FileSearchPlugin
    
    // Configuration 1: Document searcher
    inline std::unique_ptr<FileSearchPlugin> createDocumentSearcher() {
        std::vector<std::string> searchDirs = {
            QDir::homePath().toStdString() + "/Documents",
            QDir::homePath().toStdString() + "/Desktop"
        };
        
        std::vector<std::string> ignoreDirs = {
            QDir::homePath().toStdString() + "/.cache",
            QDir::homePath().toStdString() + "/.local/share/Trash"
        };
        
        return std::make_unique<FileSearchPlugin>(
            searchDirs,
            ignoreDirs, 
            2,    // max depth
            500   // max files
        );
    }
    
    // Configuration 2: Code project searcher
    inline std::unique_ptr<FileSearchPlugin> createCodeSearcher() {
        std::vector<std::string> searchDirs = {
            QDir::homePath().toStdString() + "/projects",
            QDir::homePath().toStdString() + "/dev",
            QDir::homePath().toStdString() + "/code"
        };
        
        std::vector<std::string> ignoreDirs = {
            QDir::homePath().toStdString() + "/projects/node_modules",
            QDir::homePath().toStdString() + "/projects/.git",
            QDir::homePath().toStdString() + "/projects/build",
            QDir::homePath().toStdString() + "/projects/target"
        };
        
        return std::make_unique<FileSearchPlugin>(
            searchDirs,
            ignoreDirs,
            4,     // deeper search for code projects
            1000   // more files for code projects
        );
    }
    
    // Configuration 3: Media searcher
    inline std::unique_ptr<FileSearchPlugin> createMediaSearcher() {
        std::vector<std::string> searchDirs = {
            QDir::homePath().toStdString() + "/Pictures",
            QDir::homePath().toStdString() + "/Music", 
            QDir::homePath().toStdString() + "/Videos",
            QDir::homePath().toStdString() + "/Downloads"
        };
        
        std::vector<std::string> ignoreDirs = {
            QDir::homePath().toStdString() + "/.thumbnails"
        };
        
        return std::make_unique<FileSearchPlugin>(
            searchDirs,
            ignoreDirs,
            3,     // medium depth
            2000   // lots of media files
        );
    }
}