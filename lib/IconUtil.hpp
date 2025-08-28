#pragma once

#include <QString>
#include <QUrl>
#include <QIcon>
#include <QStandardPaths>
#include <QFile>
#include <QStringList>
#include <QMimeDatabase>
#include <QMimeType>
#include <filesystem>

namespace IconUtil
{
    /**
     * Resolves an application icon using Qt's theme system and fallback searching.
     * Handles both absolute paths and theme icon names.
     * 
     * @param iconPath The icon path from a desktop entry or theme name
     * @return QUrl pointing to the resolved icon file, or empty URL if not found
     */
    QUrl resolveAppIcon(const std::string &iconPath);
    
    /**
     * Gets an appropriate icon for a file based on its MIME type.
     * Uses Qt's QMimeDatabase to determine the proper icon name, then resolves to actual file path.
     * 
     * @param filePath The filesystem path to the file
     * @return QUrl pointing to the resolved icon file, or empty URL if not found
     */
    QUrl getFileIcon(const std::filesystem::path &filePath);
    
    /**
     * Resolves a theme icon name to an actual file path.
     * 
     * @param iconName The freedesktop.org standard icon name
     * @return QUrl pointing to the resolved icon file, or empty URL if not found
     */
    QUrl resolveThemeIconToPath(const QString& iconName);
}

// Implementation
namespace IconUtil
{
    inline QUrl resolveAppIcon(const std::string &iconPath)
    {
        QString iconName = QString::fromStdString(iconPath);
        
        if (iconName.isEmpty())
            return QUrl();
        
        // If it's already a full path, use it directly
        if (iconName.startsWith('/')) {
            if (QFile::exists(iconName)) {
                return QUrl::fromLocalFile(iconName);
            }
            return QUrl();
        }
        
        // Use Qt's proper icon theme search which follows XDG spec
        QIcon icon = QIcon::fromTheme(iconName);
        bool themeFound = !icon.isNull();
        
        // Qt doesn't expose the resolved file path directly, so let's use QStandardPaths
        // to search in the proper system directories
        QStringList dataDirs = QStandardPaths::standardLocations(QStandardPaths::GenericDataLocation);
        
        // Common icon subdirectories and sizes (prioritized)
        QStringList iconSubDirs = {
            "icons/hicolor/scalable/apps",
            "icons/hicolor/48x48/apps", 
            "icons/hicolor/64x64/apps",
            "icons/hicolor/32x32/apps",
            "icons/hicolor/128x128/apps",
            "icons/Adwaita/scalable/apps",
            "icons/Adwaita/48x48/apps",
            "pixmaps"  // This should search /path/to/share/pixmaps/
        };
        
        QStringList extensions = {"", ".png", ".svg", ".xpm"};
        
        for (const QString &dataDir : dataDirs) {
            for (const QString &iconSubDir : iconSubDirs) {
                QString basePath = dataDir + "/" + iconSubDir + "/";
                for (const QString &ext : extensions) {
                    QString fullPath = basePath + iconName + ext;
                    if (QFile::exists(fullPath)) {
                        return QUrl::fromLocalFile(fullPath);
                    }
                }
            }
        }
        
        return QUrl();
    }
    
    inline QUrl getFileIcon(const std::filesystem::path &filePath)
    {
        QMimeDatabase db;
        QString fileName = QString::fromStdString(filePath.string());
        
        // Get MIME type for the file
        QMimeType mime = db.mimeTypeForFile(fileName);
        
        // Get the icon name from the MIME type
        QString iconName = mime.iconName();
        
        // If no specific icon, try the generic icon name
        if (iconName.isEmpty()) {
            iconName = mime.genericIconName();
        }
        
        // Ultimate fallback
        if (iconName.isEmpty()) {
            iconName = "text-x-generic";
        }
        
        // Now resolve the icon name to an actual file path using the same logic as desktop apps
        return resolveThemeIconToPath(iconName);
    }
    
    inline QUrl resolveThemeIconToPath(const QString& iconName)
    {
        if (iconName.isEmpty())
            return QUrl();
        
        // Use Qt's icon theme system first
        QIcon icon = QIcon::fromTheme(iconName);
        if (icon.isNull()) {
            // Try with fallback
            icon = QIcon::fromTheme("text-x-generic");
        }
        
        // Search in standard icon directories to find the actual file
        QStringList dataDirs = QStandardPaths::standardLocations(QStandardPaths::GenericDataLocation);
        
        // Icon subdirectories for MIME type icons (prioritized)
        QStringList iconSubDirs = {
            "icons/hicolor/scalable/mimetypes",
            "icons/hicolor/48x48/mimetypes",
            "icons/hicolor/32x32/mimetypes", 
            "icons/hicolor/64x64/mimetypes",
            "icons/hicolor/24x24/mimetypes",
            "icons/hicolor/16x16/mimetypes",
            "icons/Adwaita/scalable/mimetypes",
            "icons/Adwaita/48x48/mimetypes",
            "icons/Adwaita/32x32/mimetypes",
            "icons/breeze/mimetypes/22",  // KDE Plasma
            "icons/breeze-dark/mimetypes/22",
            "icons/Papirus/48x48/mimetypes",  // Popular icon theme
            "icons/elementary/mimetypes/48",  // Elementary OS
        };
        
        QStringList extensions = {".svg", ".png", ".xpm"};
        
        for (const QString& dataDir : dataDirs) {
            for (const QString& iconSubDir : iconSubDirs) {
                QString basePath = dataDir + "/" + iconSubDir + "/";
                for (const QString& ext : extensions) {
                    QString fullPath = basePath + iconName + ext;
                    if (QFile::exists(fullPath)) {
                        return QUrl::fromLocalFile(fullPath);
                    }
                }
            }
        }
        
        // Fallback: try to resolve text-x-generic if we couldn't find the specific icon
        if (iconName != "text-x-generic") {
            return resolveThemeIconToPath("text-x-generic");
        }
        
        return QUrl();
    }
}