#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include "core/events/EventManager.h"

using namespace VivoX::Core::Events;
using namespace testing;

class TestEvent : public Event {
public:
    TestEvent(const std::string& name) : Event(name) {}
    
    std::string getData() const { return m_data; }
    void setData(const std::string& data) { m_data = data; }
    
private:
    std::string m_data;
};

class TestEventHandler : public EventHandler {
public:
    TestEventHandler() : m_handleCount(0) {}
    
    bool handleEvent(const Event& event) override {
        m_lastEventName = event.getName();
        m_handleCount++;
        
        // Cast to TestEvent if possible
        const TestEvent* testEvent = dynamic_cast<const TestEvent*>(&event);
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

class EventManagerTest : public Test {
protected:
    void SetUp() override {
        // Initialize event manager
        EventManager::initialize();
    }

    void TearDown() override {
        EventManager::shutdown();
    }
};

TEST_F(EventManagerTest, InitializationTest) {
    EXPECT_TRUE(EventManager::isInitialized());
}

TEST_F(EventManagerTest, RegisterHandlerTest) {
    auto handler = std::make_shared<TestEventHandler>();
    
    // Register handler for a specific event
    EXPECT_TRUE(EventManager::registerHandler("test.event", handler));
    
    // Register handler for all events
    EXPECT_TRUE(EventManager::registerHandler("*", handler));
    
    // Register handler for a hierarchical event pattern
    EXPECT_TRUE(EventManager::registerHandler("test.*", handler));
}

TEST_F(EventManagerTest, UnregisterHandlerTest) {
    auto handler = std::make_shared<TestEventHandler>();
    
    // Register handler
    EXPECT_TRUE(EventManager::registerHandler("test.event", handler));
    
    // Unregister handler
    EXPECT_TRUE(EventManager::unregisterHandler("test.event", handler));
    
    // Unregister non-existent handler (should return false)
    EXPECT_FALSE(EventManager::unregisterHandler("test.event", handler));
}

TEST_F(EventManagerTest, DispatchEventTest) {
    auto handler = std::make_shared<TestEventHandler>();
    
    // Register handler
    EventManager::registerHandler("test.event", handler);
    
    // Create and dispatch event
    TestEvent event("test.event");
    event.setData("test data");
    
    EXPECT_TRUE(EventManager::dispatchEvent(event));
    
    // Verify handler was called
    EXPECT_EQ(handler->getHandleCount(), 1);
    EXPECT_EQ(handler->getLastEventName(), "test.event");
    EXPECT_EQ(handler->getLastEventData(), "test data");
}

TEST_F(EventManagerTest, HierarchicalEventTest) {
    auto handler = std::make_shared<TestEventHandler>();
    
    // Register handler for hierarchical pattern
    EventManager::registerHandler("test.*", handler);
    
    // Dispatch events that match the pattern
    TestEvent event1("test.event1");
    TestEvent event2("test.event2");
    TestEvent event3("test.nested.event");
    
    EXPECT_TRUE(EventManager::dispatchEvent(event1));
    EXPECT_TRUE(EventManager::dispatchEvent(event2));
    EXPECT_TRUE(EventManager::dispatchEvent(event3));
    
    // Verify handler was called for all events
    EXPECT_EQ(handler->getHandleCount(), 3);
    
    // Dispatch event that doesn't match the pattern
    TestEvent event4("other.event");
    EXPECT_TRUE(EventManager::dispatchEvent(event4));
    
    // Verify handler wasn't called for non-matching event
    EXPECT_EQ(handler->getHandleCount(), 3);
}

TEST_F(EventManagerTest, WildcardHandlerTest) {
    auto handler = std::make_shared<TestEventHandler>();
    
    // Register handler for all events
    EventManager::registerHandler("*", handler);
    
    // Dispatch various events
    TestEvent event1("test.event");
    TestEvent event2("other.event");
    TestEvent event3("completely.different.event");
    
    EXPECT_TRUE(EventManager::dispatchEvent(event1));
    EXPECT_TRUE(EventManager::dispatchEvent(event2));
    EXPECT_TRUE(EventManager::dispatchEvent(event3));
    
    // Verify handler was called for all events
    EXPECT_EQ(handler->getHandleCount(), 3);
}

TEST_F(EventManagerTest, MultipleHandlersTest) {
    auto handler1 = std::make_shared<TestEventHandler>();
    auto handler2 = std::make_shared<TestEventHandler>();
    auto handler3 = std::make_shared<TestEventHandler>();
    
    // Register multiple handlers for the same event
    EventManager::registerHandler("test.event", handler1);
    EventManager::registerHandler("test.event", handler2);
    EventManager::registerHandler("test.event", handler3);
    
    // Dispatch event
    TestEvent event("test.event");
    EXPECT_TRUE(EventManager::dispatchEvent(event));
    
    // Verify all handlers were called
    EXPECT_EQ(handler1->getHandleCount(), 1);
    EXPECT_EQ(handler2->getHandleCount(), 1);
    EXPECT_EQ(handler3->getHandleCount(), 1);
}

TEST_F(EventManagerTest, StopPropagationTest) {
    auto handler1 = std::make_shared<TestEventHandler>();
    auto handler2 = std::make_shared<TestEventHandler>();
    auto handler3 = std::make_shared<TestEventHandler>();
    
    // Set handler2 to stop propagation
    handler2->setShouldStopPropagation(true);
    
    // Register handlers in order
    EventManager::registerHandler("test.event", handler1, 1); // Priority 1 (highest)
    EventManager::registerHandler("test.event", handler2, 2); // Priority 2
    EventManager::registerHandler("test.event", handler3, 3); // Priority 3 (lowest)
    
    // Dispatch event
    TestEvent event("test.event");
    EXPECT_TRUE(EventManager::dispatchEvent(event));
    
    // Verify only handler1 and handler2 were called (handler3 should be skipped due to stopPropagation)
    EXPECT_EQ(handler1->getHandleCount(), 1);
    EXPECT_EQ(handler2->getHandleCount(), 1);
    EXPECT_EQ(handler3->getHandleCount(), 0);
}

TEST_F(EventManagerTest, PriorityOrderTest) {
    // Use a vector to track the order of handler calls
    std::vector<int> callOrder;
    
    auto handler1 = std::make_shared<TestEventHandler>();
    auto handler2 = std::make_shared<TestEventHandler>();
    auto handler3 = std::make_shared<TestEventHandler>();
    
    // Register handlers with explicit priorities
    EventManager::registerHandler("test.event", handler1, 3); // Lowest priority (called last)
    EventManager::registerHandler("test.event", handler2, 1); // Highest priority (called first)
    EventManager::registerHandler("test.event", handler3, 2); // Medium priority (called second)
    
    // Set up a callback to track the order
    EventManager::registerPreDispatchCallback([&callOrder, handler1, handler2, handler3](const Event& event) {
        if (event.getName() == "test.event") {
            if (handler1.get() == dynamic_cast<const TestEventHandler*>(event.getCurrentHandler().get())) {
                callOrder.push_back(1);
            } else if (handler2.get() == dynamic_cast<const TestEventHandler*>(event.getCurrentHandler().get())) {
                callOrder.push_back(2);
            } else if (handler3.get() == dynamic_cast<const TestEventHandler*>(event.getCurrentHandler().get())) {
                callOrder.push_back(3);
            }
        }
        return true;
    });
    
    // Dispatch event
    TestEvent event("test.event");
    EXPECT_TRUE(EventManager::dispatchEvent(event));
    
    // Verify handlers were called in priority order (highest to lowest)
    ASSERT_EQ(callOrder.size(), 3);
    EXPECT_EQ(callOrder[0], 2); // handler2 (priority 1)
    EXPECT_EQ(callOrder[1], 3); // handler3 (priority 2)
    EXPECT_EQ(callOrder[2], 1); // handler1 (priority 3)
}

TEST_F(EventManagerTest, AsyncDispatchTest) {
    auto handler = std::make_shared<TestEventHandler>();
    
    // Register handler
    EventManager::registerHandler("test.event", handler);
    
    // Create event
    auto event = std::make_shared<TestEvent>("test.event");
    event->setData("async test");
    
    // Dispatch event asynchronously
    EXPECT_TRUE(EventManager::dispatchEventAsync(event));
    
    // Wait for event to be processed
    EventManager::waitForAsyncEvents();
    
    // Verify handler was called
    EXPECT_EQ(handler->getHandleCount(), 1);
    EXPECT_EQ(handler->getLastEventName(), "test.event");
    EXPECT_EQ(handler->getLastEventData(), "async test");
}

TEST_F(EventManagerTest, PreDispatchCallbackTest) {
    auto handler = std::make_shared<TestEventHandler>();
    
    // Register handler
    EventManager::registerHandler("test.event", handler);
    
    bool callbackCalled = false;
    
    // Register pre-dispatch callback
    EventManager::registerPreDispatchCallback([&callbackCalled](const Event& event) {
        callbackCalled = true;
        return true; // Allow event to be dispatched
    });
    
    // Dispatch event
    TestEvent event("test.event");
    EXPECT_TRUE(EventManager::dispatchEvent(event));
    
    // Verify callback was called
    EXPECT_TRUE(callbackCalled);
    
    // Verify handler was called
    EXPECT_EQ(handler->getHandleCount(), 1);
}

TEST_F(EventManagerTest, PreDispatchCallbackBlockTest) {
    auto handler = std::make_shared<TestEventHandler>();
    
    // Register handler
    EventManager::registerHandler("test.event", handler);
    
    // Register pre-dispatch callback that blocks the event
    EventManager::registerPreDispatchCallback([](const Event& event) {
        return false; // Block event from being dispatched
    });
    
    // Dispatch event
    TestEvent event("test.event");
    EXPECT_FALSE(EventManager::dispatchEvent(event));
    
    // Verify handler was not called
    EXPECT_EQ(handler->getHandleCount(), 0);
}

TEST_F(EventManagerTest, PostDispatchCallbackTest) {
    auto handler = std::make_shared<TestEventHandler>();
    
    // Register handler
    EventManager::registerHandler("test.event", handler);
    
    bool callbackCalled = false;
    
    // Register post-dispatch callback
    EventManager::registerPostDispatchCallback([&callbackCalled](const Event& event, bool dispatched) {
        callbackCalled = true;
        EXPECT_TRUE(dispatched);
        return true;
    });
    
    // Dispatch event
    TestEvent event("test.event");
    EXPECT_TRUE(EventManager::dispatchEvent(event));
    
    // Verify callback was called
    EXPECT_TRUE(callbackCalled);
}

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
