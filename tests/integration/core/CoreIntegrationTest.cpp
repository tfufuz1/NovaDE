#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include "core/logging/Logger.h"
#include "core/configuration/ConfigManager.h"
#include "core/events/EventManager.h"
#include "core/plugins/PluginLoader.h"
#include "core/services/ServiceRegistry.h"
#include "core/actions/ActionManager.h"

using namespace VivoX::Core::Logging;
using namespace VivoX::Core::Configuration;
using namespace VivoX::Core::Events;
using namespace VivoX::Core::Plugins;
using namespace VivoX::Core::Services;
using namespace VivoX::Core::Actions;
using namespace testing;

class CoreIntegrationTest : public Test {
protected:
    void SetUp() override {
        // Initialize core components in the correct order
        Logger::initialize("integration_test.log", LogLevel::DEBUG);
        ConfigManager::initialize("integration_test_config.json");
        EventManager::initialize();
        PluginLoader::initialize();
        ServiceRegistry::initialize();
        ActionManager::initialize();
        
        // Create a basic config file
        ConfigManager::setString("app.name", "VivoX Integration Test");
        ConfigManager::setString("app.version", "1.0.0");
        ConfigManager::save("integration_test_config.json");
    }

    void TearDown() override {
        // Shutdown core components in the reverse order
        ActionManager::shutdown();
        ServiceRegistry::shutdown();
        PluginLoader::shutdown();
        EventManager::shutdown();
        ConfigManager::shutdown();
        Logger::shutdown();
        
        // Clean up test files
        std::remove("integration_test.log");
        std::remove("integration_test_config.json");
    }
};

// Test event class for integration testing
class TestIntegrationEvent : public Event {
public:
    TestIntegrationEvent(const std::string& name) : Event(name) {}
    
    std::string getData() const { return m_data; }
    void setData(const std::string& data) { m_data = data; }
    
private:
    std::string m_data;
};

// Test plugin class for integration testing
class TestPlugin : public PluginInterface {
public:
    TestPlugin(const std::string& id, const std::string& name, const std::string& version)
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

// Test service class for integration testing
class TestService : public Service {
public:
    TestService(const std::string& id, const std::string& name)
        : m_id(id), m_name(name) {}
    
    std::string getId() const override { return m_id; }
    std::string getName() const override { return m_name; }
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
    std::vector<std::string> m_dependencies;
    bool m_initialized = false;
};

// Test action class for integration testing
class TestAction : public ActionInterface {
public:
    TestAction(const std::string& id, const std::string& name, const std::string& category = "")
        : m_id(id), m_name(name), m_category(category) {}
    
    std::string getId() const override { return m_id; }
    std::string getName() const override { return m_name; }
    std::string getCategory() const override { return m_category; }
    std::vector<std::string> getTags() const override { return m_tags; }
    
    bool execute(const ActionParameters& params) override {
        m_executed = true;
        m_lastParams = params;
        return true;
    }
    
    bool canUndo() const override { return m_canUndo; }
    
    bool undo() override {
        if (!m_canUndo) return false;
        m_undone = true;
        return true;
    }
    
    bool isExecuted() const { return m_executed; }
    bool isUndone() const { return m_undone; }
    
    void setCanUndo(bool canUndo) { m_canUndo = canUndo; }
    
    void addTag(const std::string& tag) {
        m_tags.push_back(tag);
    }
    
    const ActionParameters& getLastParams() const { return m_lastParams; }
    
private:
    std::string m_id;
    std::string m_name;
    std::string m_category;
    std::vector<std::string> m_tags;
    bool m_executed = false;
    bool m_undone = false;
    bool m_canUndo = false;
    ActionParameters m_lastParams;
};

// Test event handler for integration testing
class TestEventHandler : public EventHandler {
public:
    TestEventHandler() : m_handleCount(0) {}
    
    bool handleEvent(const Event& event) override {
        m_lastEventName = event.getName();
        m_handleCount++;
        
        // Cast to TestIntegrationEvent if possible
        const TestIntegrationEvent* testEvent = dynamic_cast<const TestIntegrationEvent*>(&event);
        if (testEvent) {
            m_lastEventData = testEvent->getData();
        }
        
        return m_shouldStopPropagation;
    }
    
    std::string getLastEventName() const { return m_lastEventName; }
    std::string getLastEventData() const { return m_lastEventData; }
    int getHandleCount() const { return m_handleCount; }
    
    void setShouldStopPropagation(bool stop) { m_shouldStopPropagation = stop; }
    
private:
    std::string m_lastEventName;
    std::string m_lastEventData;
    int m_handleCount;
    bool m_shouldStopPropagation = false;
};

TEST_F(CoreIntegrationTest, LoggingAndConfigIntegrationTest) {
    // Test that logging respects configuration settings
    ConfigManager::setString("logging.level", "INFO");
    
    // Apply configuration to logger
    Logger::setLogLevel(static_cast<LogLevel>(ConfigManager::getInt("logging.level", static_cast<int>(LogLevel::INFO))));
    
    // Redirect stdout to capture log output
    testing::internal::CaptureStdout();
    
    Logger::trace("This is a trace message");
    Logger::debug("This is a debug message");
    Logger::info("This is an info message");
    Logger::warning("This is a warning message");
    Logger::error("This is an error message");
    
    std::string output = testing::internal::GetCapturedStdout();
    
    // Check that only INFO and above messages were logged
    EXPECT_THAT(output, Not(HasSubstr("TRACE")));
    EXPECT_THAT(output, Not(HasSubstr("DEBUG")));
    EXPECT_THAT(output, HasSubstr("INFO"));
    EXPECT_THAT(output, HasSubstr("WARNING"));
    EXPECT_THAT(output, HasSubstr("ERROR"));
}

TEST_F(CoreIntegrationTest, EventsAndLoggingIntegrationTest) {
    // Register event handler that logs events
    auto handler = std::make_shared<TestEventHandler>();
    EventManager::registerHandler("test.event", handler);
    
    // Create and dispatch event
    TestIntegrationEvent event("test.event");
    event.setData("integration test data");
    
    // Redirect stdout to capture log output
    testing::internal::CaptureStdout();
    
    // Log before dispatch
    Logger::info("Dispatching event: " + event.getName());
    
    // Dispatch event
    EventManager::dispatchEvent(event);
    
    // Log after dispatch
    Logger::info("Event handled: " + handler->getLastEventName() + " with data: " + handler->getLastEventData());
    
    std::string output = testing::internal::GetCapturedStdout();
    
    // Check that event dispatch was logged
    EXPECT_THAT(output, HasSubstr("Dispatching event: test.event"));
    EXPECT_THAT(output, HasSubstr("Event handled: test.event with data: integration test data"));
}

TEST_F(CoreIntegrationTest, PluginsAndServicesIntegrationTest) {
    // Create plugins and services with dependencies
    auto pluginA = std::make_shared<TestPlugin>("plugin.a", "Plugin A", "1.0.0");
    auto pluginB = std::make_shared<TestPlugin>("plugin.b", "Plugin B", "1.0.0");
    pluginB->addDependency("plugin.a");
    
    auto serviceA = std::make_shared<TestService>("service.a", "Service A");
    auto serviceB = std::make_shared<TestService>("service.b", "Service B");
    serviceB->addDependency("service.a");
    
    // Register plugins and services
    PluginLoader::registerPlugin(pluginA);
    PluginLoader::registerPlugin(pluginB);
    ServiceRegistry::registerService(serviceA);
    ServiceRegistry::registerService(serviceB);
    
    // Initialize all plugins and services
    EXPECT_TRUE(PluginLoader::initializeAllPlugins());
    EXPECT_TRUE(ServiceRegistry::initializeAllServices());
    
    // Verify all plugins and services are initialized
    EXPECT_TRUE(pluginA->isInitialized());
    EXPECT_TRUE(pluginB->isInitialized());
    EXPECT_TRUE(serviceA->isInitialized());
    EXPECT_TRUE(serviceB->isInitialized());
    
    // Shutdown all plugins and services
    EXPECT_TRUE(ServiceRegistry::shutdownAllServices());
    EXPECT_TRUE(PluginLoader::shutdownAllPlugins());
    
    // Verify all plugins and services are shut down
    EXPECT_FALSE(pluginA->isInitialized());
    EXPECT_FALSE(pluginB->isInitialized());
    EXPECT_FALSE(serviceA->isInitialized());
    EXPECT_FALSE(serviceB->isInitialized());
}

TEST_F(CoreIntegrationTest, ActionsAndEventsIntegrationTest) {
    // Create action
    auto action = std::make_shared<TestAction>("test.action", "Test Action");
    ActionManager::registerAction(action);
    
    // Create event handler that executes action
    auto handler = std::make_shared<EventHandler>();
    bool actionExecuted = false;
    
    handler->setHandlerFunction([&actionExecuted](const Event& event) {
        // Execute action when event is received
        ActionParameters params;
        params.setString("eventName", event.getName());
        ActionManager::executeAction("test.action", params);
        actionExecuted = true;
        return true;
    });
    
    // Register event handler
    EventManager::registerHandler("test.event", handler);
    
    // Create and dispatch event
    TestIntegrationEvent event("test.event");
    EventManager::dispatchEvent(event);
    
    // Verify action was executed
    EXPECT_TRUE(actionExecuted);
    EXPECT_TRUE(action->isExecuted());
    EXPECT_EQ(action->getLastParams().getString("eventName"), "test.event");
}

TEST_F(CoreIntegrationTest, ConfigAndPluginsIntegrationTest) {
    // Create configuration for plugins
    ConfigManager::setString("plugins.enabled", "plugin.a,plugin.b");
    ConfigManager::setString("plugins.disabled", "plugin.c");
    
    // Create plugins
    auto pluginA = std::make_shared<TestPlugin>("plugin.a", "Plugin A", "1.0.0");
    auto pluginB = std::make_shared<TestPlugin>("plugin.b", "Plugin B", "1.0.0");
    auto pluginC = std::make_shared<TestPlugin>("plugin.c", "Plugin C", "1.0.0");
    
    // Register plugins
    PluginLoader::registerPlugin(pluginA);
    PluginLoader::registerPlugin(pluginB);
    PluginLoader::registerPlugin(pluginC);
    
    // Get enabled plugins from config
    std::string enabledPluginsStr = ConfigManager::getString("plugins.enabled", "");
    std::vector<std::string> enabledPlugins;
    
    // Split comma-separated list
    size_t pos = 0;
    std::string token;
    std::string delimiter = ",";
    std::string str = enabledPluginsStr;
    while ((pos = str.find(delimiter)) != std::string::npos) {
        token = str.substr(0, pos);
        enabledPlugins.push_back(token);
        str.erase(0, pos + delimiter.length());
    }
    if (!str.empty()) {
        enabledPlugins.push_back(str);
    }
    
    // Initialize only enabled plugins
    for (const auto& pluginId : enabledPlugins) {
        EXPECT_TRUE(PluginLoader::initializePlugin(pluginId));
    }
    
    // Verify only enabled plugins are initialized
    EXPECT_TRUE(pluginA->isInitialized());
    EXPECT_TRUE(pluginB->isInitialized());
    EXPECT_FALSE(pluginC->isInitialized());
}

TEST_F(CoreIntegrationTest, FullCoreIntegrationTest) {
    // This test verifies that all core components work together correctly
    
    // 1. Create configuration
    ConfigManager::setString("app.name", "VivoX Integration Test");
    ConfigManager::setString("logging.level", "DEBUG");
    ConfigManager::setString("events.async", "true");
    
    // 2. Configure logging based on config
    Logger::setLogLevel(static_cast<LogLevel>(ConfigManager::getInt("logging.level", static_cast<int>(LogLevel::DEBUG))));
    
    // 3. Create and register plugins
    auto plugin = std::make_shared<TestPlugin>("test.plugin", "Test Plugin", "1.0.0");
    PluginLoader::registerPlugin(plugin);
    
    // 4. Create and register services
    auto service = std::make_shared<TestService>("test.service", "Test Service");
    ServiceRegistry::registerService(service);
    
    // 5. Create and register actions
    auto action = std::make_shared<TestAction>("test.action", "Test Action");
    ActionManager::registerAction(action);
    
    // 6. Create event handler that uses all components
    auto handler = std::make_shared<EventHandler>();
    handler->setHandlerFunction([](const Event& event) {
        // Log event
        Logger::info("Received event: " + event.getName());
        
        // Get plugin
        auto plugin = PluginLoader::getPlugin("test.plugin");
        if (plugin && !plugin->isInitialized()) {
            PluginLoader::initializePlugin("test.plugin");
        }
        
        // Get service
        auto service = ServiceRegistry::getService("test.service");
        if (service && !service->isInitialized()) {
            ServiceRegistry::initializeService("test.service");
        }
        
        // Execute action
        ActionParameters params;
        params.setString("eventName", event.getName());
        ActionManager::executeAction("test.action", params);
        
        return true;
    });
    
    // Register event handler
    EventManager::registerHandler("test.event", handler);
    
    // 7. Dispatch event to trigger the integration flow
    TestIntegrationEvent event("test.event");
    EventManager::dispatchEvent(event);
    
    // 8. Verify all components were used correctly
    EXPECT_TRUE(plugin->isInitialized());
    EXPECT_TRUE(service->isInitialized());
    EXPECT_TRUE(action->isExecuted());
    EXPECT_EQ(action->getLastParams().getString("eventName"), "test.event");
}

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
