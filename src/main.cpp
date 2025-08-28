#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QIcon>
#include <QWindow>
#include <QStyleHints>
#include <QtQuickControls2/QQuickStyle>
#include <LayerShellQt/window.h>

#undef signals
#include "dmenu.hpp"
#include "files.hpp"
#define signals public

#include "fuzzy.hpp"

#include "ui/AppListModel.hpp"
#include "plugins/PluginManager.hpp"
#include "plugins/DesktopAppPlugin.hpp"
#include "plugins/FileSearchPlugin.hpp"
#include <cstdlib>
#include <string>
#include <iostream>
#include <format>
#include <vector>
#include <sstream>
#include <filesystem>

int main(int argc, char *argv[])
{
    // fuzzy::FuzzyFinder fuzzy;
    // std::string home = std::getenv("HOME");
    // std::cout << "Home Directory: " << home << '\n';
    // auto projects = files::findAllFiles(home.append("/Documents/wallpapers"), 1);
    // auto pathMatches = fuzzy.find(projects, "sun", 10);
    // std::cout << "Path fuzzy search for 'anime':\n";
    // for (const auto &match : pathMatches)
    // {
    //     std::cout << "  " << match.text << " (score: " << match.score << ")\n";
    // }
    // std::cout << "\nAll files found:\n";
    // for (const auto &p : projects)
    // {
    //     std::cout << "  " << p.string() << std::endl;
    // }

    // ACTUAL APP CODE
    QGuiApplication app(argc, argv);
    QCoreApplication::setApplicationName("waycast");

    // Initialize plugin system
    auto &pluginManager = plugins::PluginManager::instance();
    pluginManager.registerPlugin(std::make_unique<plugins::DesktopAppPlugin>());
    pluginManager.registerPlugin(std::make_unique<plugins::FileSearchPlugin>());

    // Set dark theme
    QQuickStyle::setStyle("Material");
    qputenv("QT_QUICK_CONTROLS_MATERIAL_THEME", "Dark");

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
