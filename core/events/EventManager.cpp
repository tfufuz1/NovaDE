// EventManager.cpp
#include "EventManager.h"
#include <QMetaMethod>
#include <QDebug>

namespace VivoX {
namespace Core {

EventManager::EventManager() : m_initialized(false) {
    // Objekt mit initialem Zustand erstellen
}

EventManager::~EventManager() {
    // Sicherstellen, dass alle Event-Handler deaktiviert werden
    for (auto& handlerMap : m_eventHandlers) {
        handlerMap.clear();
    }
    m_eventTypeMapping.clear();
}

bool EventManager::initialize() {
    if (m_initialized) {
        qDebug() << "EventManager is already initialized";
        return true;
    }

    // Initialisiere eventuelle interne Strukturen
    qDebug() << "EventManager initialized successfully";
    m_initialized = true;
    return true;
}

bool EventManager::registerEventHandler(const QString& eventType, QObject* handler, const char* method) {
    if (!handler || !method) {
        qWarning() << "Invalid handler or method for event type:" << eventType;
        return false;
    }

    // Überprüfen, ob die Methode existiert und aufrufbar ist
    const QMetaObject* metaObject = handler->metaObject();
    QByteArray normalizedMethod = QMetaObject::normalizedSignature(method + 1); // +1 um den führenden SIGNAL/SLOT-Indikator zu überspringen
    int methodIndex = metaObject->indexOfMethod(normalizedMethod.constData());
    
    if (methodIndex < 0) {
        qWarning() << "Method" << method << "not found in handler object for event type:" << eventType;
        return false;
    }

    // Mutexschutz für Thread-Sicherheit
    QMutexLocker locker(&m_mutex);

    // Füge den Event-Typ hinzu, falls er nicht existiert
    int eventTypeId = getEventTypeId(eventType);

    // Erweitere die Handler-Liste falls notwendig
    while (m_eventHandlers.size() <= eventTypeId) {
        m_eventHandlers.append(HandlerMap());
    }

    // Füge den Handler für diesen Event-Typ hinzu
    HandlerMethod handlerMethod;
    handlerMethod.handler = handler;
    handlerMethod.method = QByteArray(method);
    
    // Überprüfen, ob der Handler bereits für diesen Event-Typ registriert ist
    auto& handlers = m_eventHandlers[eventTypeId];
    for (const auto& existing : handlers) {
        if (existing.handler == handler && existing.method == method) {
            qDebug() << "Handler already registered for event type:" << eventType;
            return true; // Handler bereits registriert
        }
    }
    
    // Neuen Handler hinzufügen
    handlers.append(handlerMethod);
    
    qDebug() << "Registered handler for event type:" << eventType;
    return true;
}

bool EventManager::unregisterEventHandler(const QString& eventType, QObject* handler, const char* method) {
    if (!handler) {
        qWarning() << "Invalid handler for event type:" << eventType;
        return false;
    }

    QMutexLocker locker(&m_mutex);

    int eventTypeId = findEventTypeId(eventType);
    if (eventTypeId < 0 || eventTypeId >= m_eventHandlers.size()) {
        qWarning() << "Event type not found:" << eventType;
        return false;
    }

    auto& handlers = m_eventHandlers[eventTypeId];
    QMutableListIterator<HandlerMethod> it(handlers);
    
    // Wenn method null ist, entferne alle Handler für dieses Objekt
    if (!method) {
        bool removed = false;
        while (it.hasNext()) {
            it.next();
            if (it.value().handler == handler) {
                it.remove();
                removed = true;
            }
        }
        if (removed) {
            qDebug() << "Removed all handlers for object on event type:" << eventType;
        }
        return removed;
    }
    
    // Ansonsten entferne nur den spezifischen Handler
    QByteArray methodName(method);
    while (it.hasNext()) {
        it.next();
        if (it.value().handler == handler && it.value().method == methodName) {
            it.remove();
            qDebug() << "Removed specific handler for event type:" << eventType;
            return true;
        }
    }

    qWarning() << "Handler not found for event type:" << eventType;
    return false;
}

bool EventManager::emitEvent(const QString& eventType, const QVariantMap& data) {
    QMutexLocker locker(&m_mutex);

    int eventTypeId = findEventTypeId(eventType);
    if (eventTypeId < 0 || eventTypeId >= m_eventHandlers.size()) {
        // Kein Handler für diesen Event-Typ registriert
        return false;
    }

    const auto& handlers = m_eventHandlers[eventTypeId];
    if (handlers.isEmpty()) {
        // Keine Handler für diesen Event-Typ
        return false;
    }

    locker.unlock(); // Mutex freigeben für die Ausführung der Handler

    bool success = true;
    for (const auto& handlerMethod : handlers) {
        // Stellen Sie sicher, dass der Handler noch existiert (nicht gelöscht wurde)
        if (handlerMethod.handler) {
            QMetaObject::invokeMethod(
                handlerMethod.handler,
                handlerMethod.method.constData(),
                Qt::DirectConnection,
                Q_ARG(QString, eventType),
                Q_ARG(QVariantMap, data)
            );
        } else {
            success = false;
        }
    }

    return success;
}

QStringList EventManager::registeredEventTypes() const {
    QMutexLocker locker(&m_mutex);
    
    QStringList result;
    for (auto it = m_eventTypeMapping.constBegin(); it != m_eventTypeMapping.constEnd(); ++it) {
        result.append(it.key());
    }
    
    return result;
}

QList<QObject*> EventManager::handlersForEventType(const QString& eventType) const {
    QMutexLocker locker(&m_mutex);
    
    QList<QObject*> result;
    
    int eventTypeId = findEventTypeId(eventType);
    if (eventTypeId >= 0 && eventTypeId < m_eventHandlers.size()) {
        const auto& handlers = m_eventHandlers[eventTypeId];
        for (const auto& handlerMethod : handlers) {
            if (handlerMethod.handler && !result.contains(handlerMethod.handler)) {
                result.append(handlerMethod.handler);
            }
        }
    }
    
    return result;
}

int EventManager::getEventTypeId(const QString& eventType) {
    auto it = m_eventTypeMapping.find(eventType);
    if (it != m_eventTypeMapping.end()) {
        return it.value();
    }
    
    // Neuen Event-Typ erstellen
    int newId = m_eventTypeMapping.size();
    m_eventTypeMapping.insert(eventType, newId);
    return newId;
}

int EventManager::findEventTypeId(const QString& eventType) const {
    auto it = m_eventTypeMapping.find(eventType);
    if (it != m_eventTypeMapping.end()) {
        return it.value();
    }
    return -1; // Event-Typ nicht gefunden
}

} // namespace Core
} // namespace VivoX