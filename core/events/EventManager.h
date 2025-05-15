#pragma once

#include <string>
#include <functional>
#include <memory>
#include <vector>
#include <map>
#include <mutex>
#include <any>
#include <queue>
#include <thread>
#include <atomic>
#include <condition_variable>

namespace VivoX {
namespace Core {
namespace Events {

/**
 * @brief Base class for all event types
 */
class Event {
public:
    virtual ~Event() = default;
    
    /**
     * @brief Get the type of the event
     * @return Event type as string
     */
    virtual std::string getType() const = 0;
    
    /**
     * @brief Get timestamp when the event was created
     * @return Timestamp in milliseconds since epoch
     */
    int64_t getTimestamp() const { return m_timestamp; }
    
    /**
     * @brief Check if event propagation should continue
     * @return True if event propagation should continue, false to stop
     */
    bool shouldPropagate() const { return m_propagate; }
    
    /**
     * @brief Stop event propagation to other handlers
     */
    void stopPropagation() { m_propagate = false; }

protected:
    Event() : m_timestamp(std::chrono::duration_cast<std::chrono::milliseconds>(
                std::chrono::system_clock::now().time_since_epoch()).count()),
              m_propagate(true) {}
    
private:
    int64_t m_timestamp;
    bool m_propagate;
};

/**
 * @brief Generic event implementation that can carry arbitrary data
 */
class GenericEvent : public Event {
public:
    /**
     * @brief Create a new generic event
     * @param type Event type
     */
    explicit GenericEvent(const std::string& type) : m_type(type) {}
    
    /**
     * @brief Get the type of the event
     * @return Event type as string
     */
    std::string getType() const override { return m_type; }
    
    /**
     * @brief Set a data value for the event
     * @param key Data key
     * @param value Data value
     */
    template<typename T>
    void setData(const std::string& key, const T& value) {
        m_data[key] = value;
    }
    
    /**
     * @brief Get a data value from the event
     * @param key Data key
     * @param defaultValue Default value to return if key not found
     * @return Data value, or defaultValue if key not found
     */
    template<typename T>
    T getData(const std::string& key, const T& defaultValue = T()) const {
        auto it = m_data.find(key);
        if (it != m_data.end()) {
            try {
                return std::any_cast<T>(it->second);
            } catch (const std::bad_any_cast&) {
                return defaultValue;
            }
        }
        return defaultValue;
    }
    
    /**
     * @brief Check if the event has a specific data key
     * @param key Data key to check
     * @return True if the key exists, false otherwise
     */
    bool hasData(const std::string& key) const {
        return m_data.find(key) != m_data.end();
    }
    
private:
    std::string m_type;
    std::map<std::string, std::any> m_data;
};

/**
 * @brief Event handler function type
 */
using EventHandler = std::function<void(const Event&)>;

/**
 * @brief Event manager for the VivoX Desktop Environment
 * 
 * This class provides a centralized way to manage events
 * for all components of the VivoX Desktop Environment.
 * Features:
 * - Thread-safe event subscription and firing
 * - Support for synchronous and asynchronous event handling
 * - Event propagation control
 * - Hierarchical event types (e.g., "input.keyboard.keypress")
 * - Event filtering
 */
class EventManager {
public:
    /**
     * @brief Get the singleton instance of the EventManager
     * @return Shared pointer to the EventManager instance
     */
    static std::shared_ptr<EventManager> getInstance();
    
    /**
     * @brief Register an event handler for a specific event type
     * @param eventType Type of event to listen for
     * @param handler Function to call when the event is fired
     * @param priority Priority of the handler (higher values are called first)
     * @return Subscription ID that can be used to unregister the handler
     */
    int subscribe(const std::string& eventType, EventHandler handler, int priority = 0);
    
    /**
     * @brief Unregister an event handler
     * @param subscriptionId ID returned by subscribe()
     * @return True if the handler was unregistered, false if the ID was invalid
     */
    bool unsubscribe(int subscriptionId);
    
    /**
     * @brief Fire an event synchronously
     * @param event Event to fire
     */
    void fireEvent(const Event& event);
    
    /**
     * @brief Fire an event asynchronously
     * @param event Event to fire
     */
    void fireEventAsync(const Event& event);
    
    /**
     * @brief Create a new event type
     * @param eventType Type of event to create
     * @return True if the event type was created, false if it already exists
     */
    bool registerEventType(const std::string& eventType);
    
    /**
     * @brief Check if an event type exists
     * @param eventType Type of event to check
     * @return True if the event type exists, false otherwise
     */
    bool hasEventType(const std::string& eventType) const;
    
    /**
     * @brief Enable or disable asynchronous event processing
     * @param enable True to enable, false to disable
     */
    void setAsyncProcessingEnabled(bool enable);
    
    /**
     * @brief Check if asynchronous event processing is enabled
     * @return True if enabled, false otherwise
     */
    bool isAsyncProcessingEnabled() const;
    
    /**
     * @brief Wait for all pending asynchronous events to be processed
     */
    void waitForPendingEvents();

private:
    EventManager();
    ~EventManager();
    
    /**
     * @brief Process events in the async queue
     */
    void processEventQueue();
    
    /**
     * @brief Find all handlers for an event type, including parent types
     * @param eventType Event type to find handlers for
     * @return Vector of handlers sorted by priority
     */
    std::vector<EventHandler> getHandlersForEvent(const std::string& eventType);
    
    /**
     * @brief Check if an event type matches a pattern (including wildcards)
     * @param eventType Event type to check
     * @param pattern Pattern to match against (can include * wildcards)
     * @return True if the event type matches the pattern, false otherwise
     */
    bool eventTypeMatches(const std::string& eventType, const std::string& pattern) const;
    
    static std::shared_ptr<EventManager> s_instance;
    static std::mutex s_instanceMutex;
    
    struct Subscription {
        int id;
        std::string eventType;
        EventHandler handler;
        int priority;
    };
    
    std::vector<std::string> m_eventTypes;
    std::map<int, Subscription> m_subscriptions;
    int m_nextSubscriptionId;
    mutable std::mutex m_mutex;
    
    // Async event processing
    std::atomic<bool> m_asyncProcessingEnabled;
    std::atomic<bool> m_running;
    std::queue<std::shared_ptr<Event>> m_eventQueue;
    std::mutex m_queueMutex;
    std::condition_variable m_queueCondition;
    std::unique_ptr<std::thread> m_processingThread;
};

} // namespace Events
} // namespace Core
} // namespace VivoX
