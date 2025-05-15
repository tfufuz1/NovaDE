#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include "core/plugins/PluginLoader.h"
#include "core/plugins/PluginInterface.h"

using namespace VivoX::Core::Plugins;
using namespace testing;

// Mock plugin implementation
class MockPlugin : public PluginInterface {
public:
    MockPlugin(const std::string& id, const std::string& name, const std::string& version)
        : m_id(id), m_name(name), m_version(version) {}
    
    std::string getId() const override { return m_id; }
    std::string getName() const override { return m_name; }
    std::string getVersion() const override { return m_version; }
    std::vector<std::string> getDependencies() const override { return m_dependencies; }
    
    bool initialize() override {
        m_initialized = true;
        return true;
    }
    
    bool shutdown() override {
        m_initialized = false;
        return true;
    }
    
    bool isInitialized() const override { return m_initialized; }
    
    void addDependency(const std::string& dependency) {
        m_dependencies.push_back(dependency);
    }
    
private:
    std::string m_id;
    std::string m_name;
    std::string m_version;
    std::vector<std::string> m_dependencies;
    bool m_initialized = false;
};

class PluginLoaderTest : public Test {
protected:
    void SetUp() override {
        // Initialize plugin loader
        PluginLoader::initialize();
    }

    void TearDown() override {
        PluginLoader::shutdown();
    }
    
    // Helper to create and register a mock plugin
    std::shared_ptr<MockPlugin> createAndRegisterPlugin(
        const std::string& id, 
        const std::string& name, 
        const std::string& version,
        const std::vector<std::string>& dependencies = {}) {
        
        auto plugin = std::make_shared<MockPlugin>(id, name, version);
        
        // Add dependencies
        for (const auto& dep : dependencies) {
            plugin->addDependency(dep);
        }
        
        // Register plugin
        PluginLoader::registerPlugin(plugin);
        
        return plugin;
    }
};

TEST_F(PluginLoaderTest, InitializationTest) {
    EXPECT_TRUE(PluginLoader::isInitialized());
}

TEST_F(PluginLoaderTest, RegisterPluginTest) {
    auto plugin = std::make_shared<MockPlugin>("test.plugin", "Test Plugin", "1.0.0");
    
    // Register plugin
    EXPECT_TRUE(PluginLoader::registerPlugin(plugin));
    
    // Try to register the same plugin again (should fail)
    EXPECT_FALSE(PluginLoader::registerPlugin(plugin));
}

TEST_F(PluginLoaderTest, GetPluginTest) {
    auto plugin = createAndRegisterPlugin("test.plugin", "Test Plugin", "1.0.0");
    
    // Get plugin by ID
    auto retrievedPlugin = PluginLoader::getPlugin("test.plugin");
    EXPECT_NE(retrievedPlugin, nullptr);
    EXPECT_EQ(retrievedPlugin->getId(), "test.plugin");
    EXPECT_EQ(retrievedPlugin->getName(), "Test Plugin");
    EXPECT_EQ(retrievedPlugin->getVersion(), "1.0.0");
    
    // Try to get non-existent plugin
    EXPECT_EQ(PluginLoader::getPlugin("non.existent.plugin"), nullptr);
}

TEST_F(PluginLoaderTest, GetAllPluginsTest) {
    // Register multiple plugins
    auto plugin1 = createAndRegisterPlugin("plugin1", "Plugin 1", "1.0.0");
    auto plugin2 = createAndRegisterPlugin("plugin2", "Plugin 2", "1.0.0");
    auto plugin3 = createAndRegisterPlugin("plugin3", "Plugin 3", "1.0.0");
    
    // Get all plugins
    auto plugins = PluginLoader::getAllPlugins();
    
    // Verify all plugins are in the list
    EXPECT_EQ(plugins.size(), 3);
    
    // Check if all plugins are in the list
    bool hasPlugin1 = false, hasPlugin2 = false, hasPlugin3 = false;
    
    for (const auto& plugin : plugins) {
        if (plugin->getId() == "plugin1") hasPlugin1 = true;
        if (plugin->getId() == "plugin2") hasPlugin2 = true;
        if (plugin->getId() == "plugin3") hasPlugin3 = true;
    }
    
    EXPECT_TRUE(hasPlugin1);
    EXPECT_TRUE(hasPlugin2);
    EXPECT_TRUE(hasPlugin3);
}

TEST_F(PluginLoaderTest, UnregisterPluginTest) {
    auto plugin = createAndRegisterPlugin("test.plugin", "Test Plugin", "1.0.0");
    
    // Unregister plugin
    EXPECT_TRUE(PluginLoader::unregisterPlugin("test.plugin"));
    
    // Verify plugin is no longer registered
    EXPECT_EQ(PluginLoader::getPlugin("test.plugin"), nullptr);
    
    // Try to unregister non-existent plugin
    EXPECT_FALSE(PluginLoader::unregisterPlugin("non.existent.plugin"));
}

TEST_F(PluginLoaderTest, InitializePluginTest) {
    auto plugin = createAndRegisterPlugin("test.plugin", "Test Plugin", "1.0.0");
    
    // Initialize plugin
    EXPECT_TRUE(PluginLoader::initializePlugin("test.plugin"));
    
    // Verify plugin is initialized
    EXPECT_TRUE(plugin->isInitialized());
    
    // Try to initialize non-existent plugin
    EXPECT_FALSE(PluginLoader::initializePlugin("non.existent.plugin"));
}

TEST_F(PluginLoaderTest, ShutdownPluginTest) {
    auto plugin = createAndRegisterPlugin("test.plugin", "Test Plugin", "1.0.0");
    
    // Initialize plugin
    PluginLoader::initializePlugin("test.plugin");
    
    // Shutdown plugin
    EXPECT_TRUE(PluginLoader::shutdownPlugin("test.plugin"));
    
    // Verify plugin is not initialized
    EXPECT_FALSE(plugin->isInitialized());
    
    // Try to shutdown non-existent plugin
    EXPECT_FALSE(PluginLoader::shutdownPlugin("non.existent.plugin"));
}

TEST_F(PluginLoaderTest, InitializeAllPluginsTest) {
    // Register multiple plugins
    auto plugin1 = createAndRegisterPlugin("plugin1", "Plugin 1", "1.0.0");
    auto plugin2 = createAndRegisterPlugin("plugin2", "Plugin 2", "1.0.0");
    auto plugin3 = createAndRegisterPlugin("plugin3", "Plugin 3", "1.0.0");
    
    // Initialize all plugins
    EXPECT_TRUE(PluginLoader::initializeAllPlugins());
    
    // Verify all plugins are initialized
    EXPECT_TRUE(plugin1->isInitialized());
    EXPECT_TRUE(plugin2->isInitialized());
    EXPECT_TRUE(plugin3->isInitialized());
}

TEST_F(PluginLoaderTest, ShutdownAllPluginsTest) {
    // Register multiple plugins
    auto plugin1 = createAndRegisterPlugin("plugin1", "Plugin 1", "1.0.0");
    auto plugin2 = createAndRegisterPlugin("plugin2", "Plugin 2", "1.0.0");
    auto plugin3 = createAndRegisterPlugin("plugin3", "Plugin 3", "1.0.0");
    
    // Initialize all plugins
    PluginLoader::initializeAllPlugins();
    
    // Shutdown all plugins
    EXPECT_TRUE(PluginLoader::shutdownAllPlugins());
    
    // Verify all plugins are not initialized
    EXPECT_FALSE(plugin1->isInitialized());
    EXPECT_FALSE(plugin2->isInitialized());
    EXPECT_FALSE(plugin3->isInitialized());
}

TEST_F(PluginLoaderTest, DependencyResolutionTest) {
    // Create plugins with dependencies
    auto pluginA = createAndRegisterPlugin("plugin.a", "Plugin A", "1.0.0");
    auto pluginB = createAndRegisterPlugin("plugin.b", "Plugin B", "1.0.0", {"plugin.a"});
    auto pluginC = createAndRegisterPlugin("plugin.c", "Plugin C", "1.0.0", {"plugin.b"});
    auto pluginD = createAndRegisterPlugin("plugin.d", "Plugin D", "1.0.0", {"plugin.a", "plugin.c"});
    
    // Initialize all plugins
    EXPECT_TRUE(PluginLoader::initializeAllPlugins());
    
    // Verify all plugins are initialized
    EXPECT_TRUE(pluginA->isInitialized());
    EXPECT_TRUE(pluginB->isInitialized());
    EXPECT_TRUE(pluginC->isInitialized());
    EXPECT_TRUE(pluginD->isInitialized());
}

TEST_F(PluginLoaderTest, CircularDependencyDetectionTest) {
    // Create plugins with circular dependencies
    auto pluginA = createAndRegisterPlugin("plugin.a", "Plugin A", "1.0.0", {"plugin.c"});
    auto pluginB = createAndRegisterPlugin("plugin.b", "Plugin B", "1.0.0", {"plugin.a"});
    auto pluginC = createAndRegisterPlugin("plugin.c", "Plugin C", "1.0.0", {"plugin.b"});
    
    // Try to initialize all plugins (should fail due to circular dependencies)
    EXPECT_FALSE(PluginLoader::initializeAllPlugins());
    
    // Verify no plugins are initialized
    EXPECT_FALSE(pluginA->isInitialized());
    EXPECT_FALSE(pluginB->isInitialized());
    EXPECT_FALSE(pluginC->isInitialized());
}

TEST_F(PluginLoaderTest, MissingDependencyTest) {
    // Create plugin with missing dependency
    auto plugin = createAndRegisterPlugin("test.plugin", "Test Plugin", "1.0.0", {"missing.dependency"});
    
    // Try to initialize plugin (should fail due to missing dependency)
    EXPECT_FALSE(PluginLoader::initializePlugin("test.plugin"));
    
    // Verify plugin is not initialized
    EXPECT_FALSE(plugin->isInitialized());
}

TEST_F(PluginLoaderTest, PluginCallbackTest) {
    bool onRegisterCalled = false;
    bool onUnregisterCalled = false;
    bool onInitializeCalled = false;
    bool onShutdownCalled = false;
    
    // Register callbacks
    PluginLoader::registerPluginCallback(PluginCallback::OnRegister, [&onRegisterCalled](const std::string& pluginId) {
        onRegisterCalled = true;
        return true;
    });
    
    PluginLoader::registerPluginCallback(PluginCallback::OnUnregister, [&onUnregisterCalled](const std::string& pluginId) {
        onUnregisterCalled = true;
        return true;
    });
    
    PluginLoader::registerPluginCallback(PluginCallback::OnInitialize, [&onInitializeCalled](const std::string& pluginId) {
        onInitializeCalled = true;
        return true;
    });
    
    PluginLoader::registerPluginCallback(PluginCallback::OnShutdown, [&onShutdownCalled](const std::string& pluginId) {
        onShutdownCalled = true;
        return true;
    });
    
    // Create and register plugin
    auto plugin = std::make_shared<MockPlugin>("test.plugin", "Test Plugin", "1.0.0");
    PluginLoader::registerPlugin(plugin);
    
    // Verify OnRegister callback was called
    EXPECT_TRUE(onRegisterCalled);
    
    // Initialize plugin
    PluginLoader::initializePlugin("test.plugin");
    
    // Verify OnInitialize callback was called
    EXPECT_TRUE(onInitializeCalled);
    
    // Shutdown plugin
    PluginLoader::shutdownPlugin("test.plugin");
    
    // Verify OnShutdown callback was called
    EXPECT_TRUE(onShutdownCalled);
    
    // Unregister plugin
    PluginLoader::unregisterPlugin("test.plugin");
    
    // Verify OnUnregister callback was called
    EXPECT_TRUE(onUnregisterCalled);
}

TEST_F(PluginLoaderTest, TopologicalSortTest) {
    // Create plugins with dependencies in reverse order
    auto pluginD = createAndRegisterPlugin("plugin.d", "Plugin D", "1.0.0", {"plugin.b", "plugin.c"});
    auto pluginC = createAndRegisterPlugin("plugin.c", "Plugin C", "1.0.0", {"plugin.a"});
    auto pluginB = createAndRegisterPlugin("plugin.b", "Plugin B", "1.0.0", {"plugin.a"});
    auto pluginA = createAndRegisterPlugin("plugin.a", "Plugin A", "1.0.0");
    
    // Get sorted plugins
    auto sortedPlugins = PluginLoader::getSortedPlugins();
    
    // Verify order is correct (A should be first, D should be last)
    ASSERT_EQ(sortedPlugins.size(), 4);
    EXPECT_EQ(sortedPlugins[0]->getId(), "plugin.a");
    EXPECT_TRUE(sortedPlugins[3]->getId() == "plugin.d");
    
    // B and C can be in either order, but both must come after A and before D
    EXPECT_TRUE(sortedPlugins[1]->getId() == "plugin.b" || sortedPlugins[1]->getId() == "plugin.c");
    EXPECT_TRUE(sortedPlugins[2]->getId() == "plugin.b" || sortedPlugins[2]->getId() == "plugin.c");
}

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
