#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include "core/actions/ActionManager.h"

using namespace VivoX::Core::Actions;
using namespace testing;

// Mock action implementation
class MockAction : public ActionInterface {
public:
    MockAction(const std::string& id, const std::string& name, const std::string& category = "")
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

class ActionManagerTest : public Test {
protected:
    void SetUp() override {
        // Initialize action manager
        ActionManager::initialize();
    }

    void TearDown() override {
        ActionManager::shutdown();
    }
    
    // Helper to create and register a mock action
    std::shared_ptr<MockAction> createAndRegisterAction(
        const std::string& id, 
        const std::string& name,
        const std::string& category = "",
        const std::vector<std::string>& tags = {}) {
        
        auto action = std::make_shared<MockAction>(id, name, category);
        
        // Add tags
        for (const auto& tag : tags) {
            action->addTag(tag);
        }
        
        // Register action
        ActionManager::registerAction(action);
        
        return action;
    }
};

TEST_F(ActionManagerTest, InitializationTest) {
    EXPECT_TRUE(ActionManager::isInitialized());
}

TEST_F(ActionManagerTest, RegisterActionTest) {
    auto action = std::make_shared<MockAction>("test.action", "Test Action");
    
    // Register action
    EXPECT_TRUE(ActionManager::registerAction(action));
    
    // Try to register the same action again (should fail)
    EXPECT_FALSE(ActionManager::registerAction(action));
}

TEST_F(ActionManagerTest, GetActionTest) {
    auto action = createAndRegisterAction("test.action", "Test Action", "test");
    
    // Get action by ID
    auto retrievedAction = ActionManager::getAction("test.action");
    EXPECT_NE(retrievedAction, nullptr);
    EXPECT_EQ(retrievedAction->getId(), "test.action");
    EXPECT_EQ(retrievedAction->getName(), "Test Action");
    EXPECT_EQ(retrievedAction->getCategory(), "test");
    
    // Try to get non-existent action
    EXPECT_EQ(ActionManager::getAction("non.existent.action"), nullptr);
}

TEST_F(ActionManagerTest, GetAllActionsTest) {
    // Register multiple actions
    auto action1 = createAndRegisterAction("action1", "Action 1");
    auto action2 = createAndRegisterAction("action2", "Action 2");
    auto action3 = createAndRegisterAction("action3", "Action 3");
    
    // Get all actions
    auto actions = ActionManager::getAllActions();
    
    // Verify all actions are in the list
    EXPECT_EQ(actions.size(), 3);
    
    // Check if all actions are in the list
    bool hasAction1 = false, hasAction2 = false, hasAction3 = false;
    
    for (const auto& action : actions) {
        if (action->getId() == "action1") hasAction1 = true;
        if (action->getId() == "action2") hasAction2 = true;
        if (action->getId() == "action3") hasAction3 = true;
    }
    
    EXPECT_TRUE(hasAction1);
    EXPECT_TRUE(hasAction2);
    EXPECT_TRUE(hasAction3);
}

TEST_F(ActionManagerTest, UnregisterActionTest) {
    auto action = createAndRegisterAction("test.action", "Test Action");
    
    // Unregister action
    EXPECT_TRUE(ActionManager::unregisterAction("test.action"));
    
    // Verify action is no longer registered
    EXPECT_EQ(ActionManager::getAction("test.action"), nullptr);
    
    // Try to unregister non-existent action
    EXPECT_FALSE(ActionManager::unregisterAction("non.existent.action"));
}

TEST_F(ActionManagerTest, ExecuteActionTest) {
    auto action = createAndRegisterAction("test.action", "Test Action");
    
    // Execute action
    ActionParameters params;
    params.setString("param1", "value1");
    params.setInt("param2", 42);
    
    EXPECT_TRUE(ActionManager::executeAction("test.action", params));
    
    // Verify action was executed
    EXPECT_TRUE(action->isExecuted());
    
    // Verify parameters were passed correctly
    EXPECT_EQ(action->getLastParams().getString("param1"), "value1");
    EXPECT_EQ(action->getLastParams().getInt("param2"), 42);
    
    // Try to execute non-existent action
    EXPECT_FALSE(ActionManager::executeAction("non.existent.action", params));
}

TEST_F(ActionManagerTest, UndoActionTest) {
    auto action = createAndRegisterAction("test.action", "Test Action");
    action->setCanUndo(true);
    
    // Execute action
    ActionParameters params;
    ActionManager::executeAction("test.action", params);
    
    // Undo action
    EXPECT_TRUE(ActionManager::undoAction("test.action"));
    
    // Verify action was undone
    EXPECT_TRUE(action->isUndone());
    
    // Try to undo non-existent action
    EXPECT_FALSE(ActionManager::undoAction("non.existent.action"));
    
    // Try to undo action that can't be undone
    auto nonUndoableAction = createAndRegisterAction("non.undoable.action", "Non-Undoable Action");
    nonUndoableAction->setCanUndo(false);
    
    ActionManager::executeAction("non.undoable.action", params);
    EXPECT_FALSE(ActionManager::undoAction("non.undoable.action"));
}

TEST_F(ActionManagerTest, GetActionsByTagTest) {
    // Register actions with tags
    auto action1 = createAndRegisterAction("action1", "Action 1", "category1", {"tag1", "tag2"});
    auto action2 = createAndRegisterAction("action2", "Action 2", "category1", {"tag2", "tag3"});
    auto action3 = createAndRegisterAction("action3", "Action 3", "category2", {"tag1", "tag3"});
    
    // Get actions by tag
    auto actionsWithTag1 = ActionManager::getActionsByTag("tag1");
    auto actionsWithTag2 = ActionManager::getActionsByTag("tag2");
    auto actionsWithTag3 = ActionManager::getActionsByTag("tag3");
    auto actionsWithNonExistentTag = ActionManager::getActionsByTag("non.existent.tag");
    
    // Verify correct actions are returned
    EXPECT_EQ(actionsWithTag1.size(), 2);
    EXPECT_EQ(actionsWithTag2.size(), 2);
    EXPECT_EQ(actionsWithTag3.size(), 2);
    EXPECT_EQ(actionsWithNonExistentTag.size(), 0);
    
    // Check specific actions in tag1 list
    bool hasAction1InTag1 = false, hasAction3InTag1 = false;
    
    for (const auto& action : actionsWithTag1) {
        if (action->getId() == "action1") hasAction1InTag1 = true;
        if (action->getId() == "action3") hasAction3InTag1 = true;
    }
    
    EXPECT_TRUE(hasAction1InTag1);
    EXPECT_TRUE(hasAction3InTag1);
}

TEST_F(ActionManagerTest, GetActionsByCategoryTest) {
    // Register actions with categories
    auto action1 = createAndRegisterAction("action1", "Action 1", "category1");
    auto action2 = createAndRegisterAction("action2", "Action 2", "category1");
    auto action3 = createAndRegisterAction("action3", "Action 3", "category2");
    
    // Get actions by category
    auto actionsInCategory1 = ActionManager::getActionsByCategory("category1");
    auto actionsInCategory2 = ActionManager::getActionsByCategory("category2");
    auto actionsInNonExistentCategory = ActionManager::getActionsByCategory("non.existent.category");
    
    // Verify correct actions are returned
    EXPECT_EQ(actionsInCategory1.size(), 2);
    EXPECT_EQ(actionsInCategory2.size(), 1);
    EXPECT_EQ(actionsInNonExistentCategory.size(), 0);
    
    // Check specific actions in category1 list
    bool hasAction1InCategory1 = false, hasAction2InCategory1 = false;
    
    for (const auto& action : actionsInCategory1) {
        if (action->getId() == "action1") hasAction1InCategory1 = true;
        if (action->getId() == "action2") hasAction2InCategory1 = true;
    }
    
    EXPECT_TRUE(hasAction1InCategory1);
    EXPECT_TRUE(hasAction2InCategory1);
}

TEST_F(ActionManagerTest, ActionHistoryTest) {
    // Register actions
    auto action1 = createAndRegisterAction("action1", "Action 1");
    auto action2 = createAndRegisterAction("action2", "Action 2");
    
    // Execute actions
    ActionParameters params;
    ActionManager::executeAction("action1", params);
    ActionManager::executeAction("action2", params);
    
    // Get action history
    auto history = ActionManager::getActionHistory();
    
    // Verify history contains the executed actions in correct order
    ASSERT_EQ(history.size(), 2);
    EXPECT_EQ(history[0].actionId, "action1");
    EXPECT_EQ(history[1].actionId, "action2");
}

TEST_F(ActionManagerTest, ClearActionHistoryTest) {
    // Register and execute actions
    auto action = createAndRegisterAction("test.action", "Test Action");
    ActionParameters params;
    ActionManager::executeAction("test.action", params);
    
    // Verify history contains the executed action
    EXPECT_EQ(ActionManager::getActionHistory().size(), 1);
    
    // Clear history
    ActionManager::clearActionHistory();
    
    // Verify history is empty
    EXPECT_EQ(ActionManager::getActionHistory().size(), 0);
}

TEST_F(ActionManagerTest, ActionCallbackTest) {
    bool beforeExecuteCalled = false;
    bool afterExecuteCalled = false;
    bool beforeUndoCalled = false;
    bool afterUndoCalled = false;
    
    // Register callbacks
    ActionManager::registerActionCallback(ActionCallback::BeforeExecute, [&beforeExecuteCalled](const std::string& actionId, const ActionParameters& params) {
        beforeExecuteCalled = true;
        return true;
    });
    
    ActionManager::registerActionCallback(ActionCallback::AfterExecute, [&afterExecuteCalled](const std::string& actionId, const ActionParameters& params, bool success) {
        afterExecuteCalled = true;
        return true;
    });
    
    ActionManager::registerActionCallback(ActionCallback::BeforeUndo, [&beforeUndoCalled](const std::string& actionId) {
        beforeUndoCalled = true;
        return true;
    });
    
    ActionManager::registerActionCallback(ActionCallback::AfterUndo, [&afterUndoCalled](const std::string& actionId, bool success) {
        afterUndoCalled = true;
        return true;
    });
    
    // Create and register action
    auto action = createAndRegisterAction("test.action", "Test Action");
    action->setCanUndo(true);
    
    // Execute action
    ActionParameters params;
    ActionManager::executeAction("test.action", params);
    
    // Verify callbacks were called
    EXPECT_TRUE(beforeExecuteCalled);
    EXPECT_TRUE(afterExecuteCalled);
    
    // Undo action
    ActionManager::undoAction("test.action");
    
    // Verify callbacks were called
    EXPECT_TRUE(beforeUndoCalled);
    EXPECT_TRUE(afterUndoCalled);
}

TEST_F(ActionManagerTest, ParameterValidationTest) {
    // Create action with parameter validation
    auto action = std::make_shared<MockAction>("test.action", "Test Action");
    
    // Register action
    ActionManager::registerAction(action);
    
    // Register parameter validator
    ActionManager::registerParameterValidator("test.action", [](const ActionParameters& params) {
        // Require "required_param" to be present
        if (!params.hasParam("required_param")) {
            return std::make_pair(false, "Missing required parameter: required_param");
        }
        return std::make_pair(true, "");
    });
    
    // Try to execute action with invalid parameters
    ActionParameters invalidParams;
    EXPECT_FALSE(ActionManager::executeAction("test.action", invalidParams));
    
    // Execute action with valid parameters
    ActionParameters validParams;
    validParams.setString("required_param", "value");
    EXPECT_TRUE(ActionManager::executeAction("test.action", validParams));
}

TEST_F(ActionManagerTest, SearchActionsTest) {
    // Register actions
    auto action1 = createAndRegisterAction("action.one", "First Action", "category1", {"tag1"});
    auto action2 = createAndRegisterAction("action.two", "Second Action", "category2", {"tag2"});
    auto action3 = createAndRegisterAction("other.three", "Third Action", "category1", {"tag1"});
    
    // Search by ID pattern
    auto actionsByIdPattern = ActionManager::searchActions("action.*");
    EXPECT_EQ(actionsByIdPattern.size(), 2);
    
    // Search by name pattern
    auto actionsByNamePattern = ActionManager::searchActions("*Action");
    EXPECT_EQ(actionsByNamePattern.size(), 3);
    
    // Search by exact name
    auto actionsByExactName = ActionManager::searchActions("First Action", true);
    EXPECT_EQ(actionsByExactName.size(), 1);
    EXPECT_EQ(actionsByExactName[0]->getId(), "action.one");
    
    // Search by category
    auto actionsByCategory = ActionManager::searchActions("", false, "category1");
    EXPECT_EQ(actionsByCategory.size(), 2);
    
    // Search by tag
    auto actionsByTag = ActionManager::searchActions("", false, "", "tag1");
    EXPECT_EQ(actionsByTag.size(), 2);
    
    // Combined search
    auto actionsCombined = ActionManager::searchActions("*Action", false, "category1", "tag1");
    EXPECT_EQ(actionsCombined.size(), 2);
}

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
