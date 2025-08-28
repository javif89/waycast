#pragma once
#include "files.hpp"
#include <glib.h>
#include <gio/gio.h>
#include <gio/gappinfo.h>
#include <gio-unix-2.0/gio/gdesktopappinfo.h>
#include <string>
#include <vector>
#include <iostream>

namespace dmenu
{
    class DesktopEntry
    {
    public:
        std::string id;
        std::string name;
        std::string icon_path;
        std::string exec;
        bool display = false;
        DesktopEntry(std::string path)
        {
            GDesktopAppInfo *info = g_desktop_app_info_new_from_filename(path.c_str());
            if (!info) {
                std::cerr << "Failed to create desktop app info for: " << path << std::endl;
                return;
            }
            
            GAppInfo *app = G_APP_INFO(info);
            if (!app) {
                std::cerr << "Failed to get app info for: " << path << std::endl;
                g_object_unref(info);
                return;
            }

            const char *app_id = g_app_info_get_id(app);
            if (app_id) id = app_id;
            
            const char *app_name = g_app_info_get_name(app);
            if (app_name) name = app_name;
            
            GIcon *icon = g_app_info_get_icon(app);
            if (icon) {
                char* icon_str = g_icon_to_string(icon);
                if (icon_str) {
                    icon_path = icon_str;
                    g_free(icon_str);
                }
            }
                
            const char *ex = g_app_info_get_executable(app);
            if (ex)
                exec = ex;
                
            display = g_desktop_app_info_get_boolean(info, "NoDisplay") ? false : true;
            g_object_unref(info);
        }

        std::string iconPath()
        {
            return icon_path;
        }
    };

    std::vector<std::string> split(std::string s, char delimiter)
    {
        std::vector<std::string> items;
        std::string line;
        std::stringstream ss(s);

        while (std::getline(ss, line, delimiter))
        {
            items.push_back(line);
        }

        return items;
    }

    using DEVec = std::unique_ptr<std::vector<DesktopEntry>>;
    DEVec get_dmenu_app_data()
    {
        DEVec out = std::make_unique<std::vector<DesktopEntry>>();
        const char* env_dirs = std::getenv("XDG_DATA_DIRS");
        if (!env_dirs) {
            std::cerr << "XDG_DATA_DIRS environment variable not set" << std::endl;
            return out;
        }
        std::string dataDirs = env_dirs;
        std::vector<std::string> paths = split(dataDirs, ':');
        for (std::string &p : paths)
            p.append("/applications");

        for (std::string &p : paths)
        {
            std::vector<std::filesystem::path> desktopFiles = files::findFilesWithExtension(p, ".desktop");

            for (const auto &dfile : desktopFiles)
            {
                out->emplace_back(dfile.string());
            }
        }

        return out;
    }

}