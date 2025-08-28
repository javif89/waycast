#pragma once

#include "ListItem.hpp"

#undef signals
#include "dmenu.hpp"
#define signals public

class DesktopAppListItem : public ListItem
{
public:
    explicit DesktopAppListItem(const dmenu::DesktopEntry &entry);

    // ListItem interface
    QString name() const override;
    QString description() const override;
    QUrl iconUrl() const override;
    void execute() override;
    QString itemType() const override;

private:
    dmenu::DesktopEntry m_entry;
};