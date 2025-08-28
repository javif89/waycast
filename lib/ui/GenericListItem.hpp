#pragma once

#include "ListItem.hpp"
#include <functional>
#include <QProcess>

// Generic implementation that plugin developers can use without writing boilerplate
class GenericListItem : public ListItem
{
public:
    GenericListItem(const QString& name, 
                   const QString& description, 
                   const QUrl& iconUrl, 
                   const QString& itemType,
                   std::function<void()> executeFunc = nullptr)
        : m_name(name)
        , m_description(description) 
        , m_iconUrl(iconUrl)
        , m_itemType(itemType)
        , m_executeFunc(executeFunc)
    {
    }

    QString name() const override { return m_name; }
    QString description() const override { return m_description; }
    QUrl iconUrl() const override { return m_iconUrl; }
    QString itemType() const override { return m_itemType; }
    
    void execute() override 
    { 
        if (m_executeFunc) {
            m_executeFunc();
        }
    }

    // Allow updating properties after construction
    void setName(const QString& name) { m_name = name; }
    void setDescription(const QString& description) { m_description = description; }
    void setIconUrl(const QUrl& iconUrl) { m_iconUrl = iconUrl; }
    void setExecuteFunction(std::function<void()> func) { m_executeFunc = func; }

private:
    QString m_name;
    QString m_description;
    QUrl m_iconUrl;
    QString m_itemType;
    std::function<void()> m_executeFunc;
};

// Convenience factory functions for common use cases
namespace ListItems {
    
    // Create an application-style item
    inline ListItemPtr createApplication(const QString& name, 
                                       const QString& exec, 
                                       const QUrl& icon = QUrl()) 
    {
        return std::make_shared<GenericListItem>(
            name, exec, icon, "app",
            [exec]() {
                // Basic app execution - plugins can override for complex logic
                QProcess::startDetached("/bin/sh", QStringList() << "-c" << exec);
            }
        );
    }
    
    // Create a file-style item  
    inline ListItemPtr createFile(const QString& filename,
                                const QString& path,
                                const QUrl& icon = QUrl())
    {
        return std::make_shared<GenericListItem>(
            filename, path, icon, "file",
            [path]() {
                QProcess::startDetached("xdg-open", QStringList() << path);
            }
        );
    }
    
    // Create a generic item with custom action
    inline ListItemPtr createItem(const QString& name,
                                const QString& description,
                                const QString& itemType,
                                std::function<void()> action = nullptr,
                                const QUrl& icon = QUrl())
    {
        return std::make_shared<GenericListItem>(name, description, icon, itemType, action);
    }
}