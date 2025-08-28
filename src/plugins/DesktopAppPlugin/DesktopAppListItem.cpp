#include "DesktopAppListItem.hpp"
#include <QProcess>
#include <QRegularExpression>
#include <QIcon>
#include <QStandardPaths>
#include <QFile>
#include <QDebug>

DesktopAppListItem::DesktopAppListItem(const dmenu::DesktopEntry &entry)
    : m_entry(entry)
{
}

QString DesktopAppListItem::name() const
{
    return QString::fromStdString(m_entry.name);
}

QString DesktopAppListItem::description() const
{
    // Could add support for Comment field from desktop entry later
    return QString::fromStdString(m_entry.exec);
}

QUrl DesktopAppListItem::iconUrl() const
{
    return resolveIconUrl(m_entry.icon_path);
}

void DesktopAppListItem::execute()
{
    // Parse exec command (remove %f, %u, %F, %U field codes if present)
    QString command = QString::fromStdString(m_entry.exec);
    QRegularExpression fieldCodes("%[fuFU]");
    command = command.replace(fieldCodes, "").trimmed();
    
    // Use nohup and redirect output to /dev/null for proper detachment
    QString detachedCommand = QString("nohup %1 >/dev/null 2>&1 &").arg(command);
    QProcess::startDetached("/bin/sh", QStringList() << "-c" << detachedCommand);
}

QString DesktopAppListItem::itemType() const
{
    return "app";
}

QUrl DesktopAppListItem::resolveIconUrl(const std::string &iconPath) const
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