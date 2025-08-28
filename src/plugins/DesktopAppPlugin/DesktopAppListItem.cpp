#include "DesktopAppListItem.hpp"
#include "IconUtil.hpp"
#include <QProcess>
#include <QRegularExpression>
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
    return IconUtil::resolveAppIcon(m_entry.icon_path);
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

