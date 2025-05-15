#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include "core/configuration/ConfigManager.h"
#include <fstream>

using namespace VivoX::Core::Configuration;
using namespace testing;

class ConfigManagerTest : public Test {
protected:
    void SetUp() override {
        // Create a test config file
        std::ofstream configFile("test_config.json");
        configFile << R"({
            "app": {
                "name": "VivoX",
                "version": "1.0.0"
            },
            "ui": {
                "theme": "dark",
                "scale": 1.2,
                "fonts": {
                    "main": "Noto Sans",
                    "monospace": "Noto Mono"
                }
            },
            "system": {
                "logging": {
                    "level": "info",
                    "file": "vivox.log"
                },
                "performance": {
                    "enable_gpu": true,
                    "max_fps": 60
                }
            }
        })";
        configFile.close();
        
        // Initialize config manager
        ConfigManager::initialize("test_config.json");
    }

    void TearDown() override {
        ConfigManager::shutdown();
        // Clean up test config file
        std::remove("test_config.json");
    }
};

TEST_F(ConfigManagerTest, InitializationTest) {
    EXPECT_TRUE(ConfigManager::isInitialized());
}

TEST_F(ConfigManagerTest, GetStringTest) {
    EXPECT_EQ(ConfigManager::getString("app.name"), "VivoX");
    EXPECT_EQ(ConfigManager::getString("ui.theme"), "dark");
    EXPECT_EQ(ConfigManager::getString("ui.fonts.main"), "Noto Sans");
    EXPECT_EQ(ConfigManager::getString("ui.fonts.monospace"), "Noto Mono");
    EXPECT_EQ(ConfigManager::getString("system.logging.level"), "info");
    EXPECT_EQ(ConfigManager::getString("system.logging.file"), "vivox.log");
    
    // Test default value for non-existent key
    EXPECT_EQ(ConfigManager::getString("non.existent.key", "default"), "default");
}

TEST_F(ConfigManagerTest, GetIntTest) {
    EXPECT_EQ(ConfigManager::getInt("system.performance.max_fps"), 60);
    
    // Test default value for non-existent key
    EXPECT_EQ(ConfigManager::getInt("non.existent.key", 42), 42);
    
    // Test default value for wrong type
    EXPECT_EQ(ConfigManager::getInt("app.name", 42), 42);
}

TEST_F(ConfigManagerTest, GetDoubleTest) {
    EXPECT_DOUBLE_EQ(ConfigManager::getDouble("ui.scale"), 1.2);
    
    // Test default value for non-existent key
    EXPECT_DOUBLE_EQ(ConfigManager::getDouble("non.existent.key", 3.14), 3.14);
    
    // Test default value for wrong type
    EXPECT_DOUBLE_EQ(ConfigManager::getDouble("app.name", 3.14), 3.14);
}

TEST_F(ConfigManagerTest, GetBoolTest) {
    EXPECT_TRUE(ConfigManager::getBool("system.performance.enable_gpu"));
    
    // Test default value for non-existent key
    EXPECT_FALSE(ConfigManager::getBool("non.existent.key", false));
    
    // Test default value for wrong type
    EXPECT_TRUE(ConfigManager::getBool("app.name", true));
}

TEST_F(ConfigManagerTest, SetValueTest) {
    // Set new values
    ConfigManager::setString("app.name", "VivoX Desktop");
    ConfigManager::setInt("system.performance.max_fps", 120);
    ConfigManager::setDouble("ui.scale", 1.5);
    ConfigManager::setBool("system.performance.enable_gpu", false);
    
    // Verify new values
    EXPECT_EQ(ConfigManager::getString("app.name"), "VivoX Desktop");
    EXPECT_EQ(ConfigManager::getInt("system.performance.max_fps"), 120);
    EXPECT_DOUBLE_EQ(ConfigManager::getDouble("ui.scale"), 1.5);
    EXPECT_FALSE(ConfigManager::getBool("system.performance.enable_gpu"));
    
    // Set value for non-existent key (should create the key)
    ConfigManager::setString("new.key", "new value");
    EXPECT_EQ(ConfigManager::getString("new.key"), "new value");
}

TEST_F(ConfigManagerTest, HasKeyTest) {
    EXPECT_TRUE(ConfigManager::hasKey("app.name"));
    EXPECT_TRUE(ConfigManager::hasKey("ui.theme"));
    EXPECT_TRUE(ConfigManager::hasKey("system.logging.level"));
    
    EXPECT_FALSE(ConfigManager::hasKey("non.existent.key"));
    EXPECT_FALSE(ConfigManager::hasKey("app.non_existent"));
}

TEST_F(ConfigManagerTest, RemoveKeyTest) {
    EXPECT_TRUE(ConfigManager::hasKey("app.name"));
    
    // Remove key
    ConfigManager::removeKey("app.name");
    
    EXPECT_FALSE(ConfigManager::hasKey("app.name"));
    
    // Try to remove non-existent key (should not crash)
    ConfigManager::removeKey("non.existent.key");
}

TEST_F(ConfigManagerTest, SaveAndLoadTest) {
    // Modify some values
    ConfigManager::setString("app.name", "VivoX Desktop");
    ConfigManager::setInt("system.performance.max_fps", 120);
    
    // Save to a new file
    ConfigManager::save("test_config_modified.json");
    
    // Load the new file
    ConfigManager::load("test_config_modified.json");
    
    // Verify values were saved and loaded correctly
    EXPECT_EQ(ConfigManager::getString("app.name"), "VivoX Desktop");
    EXPECT_EQ(ConfigManager::getInt("system.performance.max_fps"), 120);
    
    // Clean up
    std::remove("test_config_modified.json");
}

TEST_F(ConfigManagerTest, HierarchicalKeysTest) {
    // Test deep hierarchical keys
    ConfigManager::setString("very.deep.hierarchical.key", "deep value");
    EXPECT_EQ(ConfigManager::getString("very.deep.hierarchical.key"), "deep value");
    
    // Test partial hierarchy
    EXPECT_TRUE(ConfigManager::hasKey("very.deep.hierarchical"));
    EXPECT_TRUE(ConfigManager::hasKey("very.deep"));
    EXPECT_TRUE(ConfigManager::hasKey("very"));
}

TEST_F(ConfigManagerTest, SystemDefaultsTest) {
    // Test system defaults
    ConfigManager::setSystemDefault("test.system.default", "system value");
    
    // User value should override system default
    ConfigManager::setString("test.system.default", "user value");
    EXPECT_EQ(ConfigManager::getString("test.system.default"), "user value");
    
    // Remove user value, should fall back to system default
    ConfigManager::removeKey("test.system.default");
    EXPECT_EQ(ConfigManager::getString("test.system.default"), "system value");
}

TEST_F(ConfigManagerTest, FileWatchingTest) {
    // This is a basic test to ensure file watching doesn't crash
    // In a real test, we would use a mock to verify callbacks are called
    
    bool callbackCalled = false;
    ConfigManager::addChangeCallback([&callbackCalled](const std::string& key) {
        callbackCalled = true;
    });
    
    // Simulate file change by saving
    ConfigManager::save("test_config.json");
    
    // In a real scenario, we would wait for the file system watcher to detect the change
    // For this test, we'll just verify the callback registration doesn't crash
    SUCCEED();
}

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
