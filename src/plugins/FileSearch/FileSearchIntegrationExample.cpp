// Example showing how to integrate FileSearchPlugin into main.cpp
// This is not compiled - it's just a reference for how to use the plugin

/*
#include "plugins/FileSearchPlugin.hpp"
#include "plugins/FileSearchExample.hpp"

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);
    QCoreApplication::setApplicationName("waycast");

    // Initialize plugin system
    auto& pluginManager = plugins::PluginManager::instance();
    
    // Register desktop app plugin
    pluginManager.registerPlugin(std::make_unique<plugins::DesktopAppPlugin>());
    
    // Register file search plugins with different configurations
    
    // Option 1: Use default configuration
    pluginManager.registerPlugin(std::make_unique<plugins::FileSearchPlugin>());
    
    // Option 2: Use custom configuration
    std::vector<std::string> customSearchDirs = {
        "/home/user/Documents", 
        "/home/user/Projects"
    };
    std::vector<std::string> customIgnoreDirs = {
        "/home/user/.cache",
        "/home/user/Projects/node_modules"
    };
    pluginManager.registerPlugin(std::make_unique<plugins::FileSearchPlugin>(
        customSearchDirs,
        customIgnoreDirs,
        3,    // max depth
        1000  // max files
    ));
    
    // Option 3: Use predefined configurations
    pluginManager.registerPlugin(plugins::createDocumentSearcher());
    pluginManager.registerPlugin(plugins::createMediaSearcher());
    
    // Continue with rest of Qt application setup...
    // ...
    
    return app.exec();
}
*/