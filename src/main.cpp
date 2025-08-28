#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QIcon>
#include <QWindow>
#include <LayerShellQt/window.h>
#include "ui/AppListModel.hpp"
#include "dmenu.hpp"
#include "files.hpp"
#include <cstdlib>
#include <string>
#include <iostream>
#include <format>
#include <vector>
#include <sstream>
#include <filesystem>

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

int main(int argc, char *argv[])
{
    // dmenu::DEVec apps = dmenu::get_dmenu_app_data();
    // for (auto &app : *apps.get())
    // {
    //     std::cout << app.iconPath() << std::endl;
    //     // std::cout << std::format("---\nName: {}\nID: {}\nIcon: {}\nExec: {}\nDisp: {}\n---\n", app.id, app.name, app.iconPath(), app.exec, app.display ? "yes" : "no");
    // }

    QGuiApplication app(argc, argv);
    QCoreApplication::setApplicationName("waycast");

    // Enable system theme support
    app.setDesktopSettingsAware(true);

    QQmlApplicationEngine engine;

    // Register the AppListModel type with QML
    qmlRegisterType<AppListModel>("WayCast", 1, 0, "AppListModel");

    // Set up layer shell before creating any windows
    QObject::connect(&engine, &QQmlApplicationEngine::objectCreationFailed, &app, []()
                     { QCoreApplication::exit(-1); });

    engine.loadFromModule("WayCast", "Main");

    // Get the root objects and configure layer shell
    auto rootObjects = engine.rootObjects();
    if (!rootObjects.isEmpty())
    {
        QWindow *window = qobject_cast<QWindow *>(rootObjects.first());
        if (window)
        {
            LayerShellQt::Window *layerWindow = LayerShellQt::Window::get(window);
            if (layerWindow)
            {
                layerWindow->setLayer(LayerShellQt::Window::LayerTop);
                layerWindow->setAnchors({});
                layerWindow->setKeyboardInteractivity(LayerShellQt::Window::KeyboardInteractivityOnDemand);

                // Now show the window after layer shell is configured
                window->show();
            }
        }
    }
    return app.exec();
}
