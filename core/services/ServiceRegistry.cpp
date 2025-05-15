// ServiceRegistry.cpp
#include "ServiceRegistry.h"
#include <QDebug>

namespace VivoX {
namespace Core {

ServiceRegistry* ServiceRegistry::s_instance = nullptr;
QMutex ServiceRegistry::s_instanceMutex;

ServiceRegistry* ServiceRegistry::getInstance() {
    QMutexLocker locker(&s_instanceMutex);
    
    if (!s_instance) {
        s_instance = new ServiceRegistry();
    }
    
    return s_instance;
}

ServiceRegistry::ServiceRegistry() : m_initialized(false) {
    // Initialer Zustand des Service-Registry
}

ServiceRegistry::~ServiceRegistry() {
    // Stoppe und entferne alle registrierten Services
    shutdownAllServices();
    m_services.clear();
}

bool ServiceRegistry::initialize() {
    QMutexLocker locker(&m_mutex);
    
    if (m_initialized) {
        qDebug() << "ServiceRegistry already initialized";
        return true;
    }
    
    qDebug() << "ServiceRegistry initialized successfully";
    m_initialized = true;
    return true;
}

bool ServiceRegistry::registerService(ServiceInterface* service) {
    if (!service) {
        qWarning() << "Cannot register null service";
        return false;
    }
    
    QMutexLocker locker(&m_mutex);
    
    QString serviceId = service->serviceId();
    if (m_services.contains(serviceId)) {
        qWarning() << "Service already registered with ID:" << serviceId;
        return false;
    }
    
    m_services.insert(serviceId, service);
    qDebug() << "Service registered successfully:" << serviceId;
    return true;
}

bool ServiceRegistry::unregisterService(const QString& serviceId) {
    QMutexLocker locker(&m_mutex);
    
    auto it = m_services.find(serviceId);
    if (it == m_services.end()) {
        qWarning() << "Service not found for unregistration:" << serviceId;
        return false;
    }
    
    // Shutdown service if initialized
    ServiceInterface* service = it.value();
    if (service->isInitialized()) {
        service->shutdown();
    }
    
    m_services.remove(serviceId);
    qDebug() << "Service unregistered successfully:" << serviceId;
    return true;
}

ServiceInterface* ServiceRegistry::getService(const QString& serviceId) const {
    QMutexLocker locker(&m_mutex);
    
    auto it = m_services.find(serviceId);
    if (it == m_services.end()) {
        return nullptr;
    }
    
    return it.value();
}

QList<QString> ServiceRegistry::getServiceIds() const {
    QMutexLocker locker(&m_mutex);
    return m_services.keys();
}

bool ServiceRegistry::hasService(const QString& serviceId) const {
    QMutexLocker locker(&m_mutex);
    return m_services.contains(serviceId);
}

bool ServiceRegistry::initializeAllServices() {
    QMutexLocker locker(&m_mutex);
    
    bool allSuccessful = true;
    QList<QString> failedServices;
    
    // Analysiere Abhängigkeiten und erstelle eine Initialisierungsreihenfolge
    QList<QString> initOrder = getInitializationOrder();
    
    // Initialisiere Services in der berechneten Reihenfolge
    for (const QString& serviceId : initOrder) {
        if (!initializeService(serviceId)) {
            allSuccessful = false;
            failedServices.append(serviceId);
        }
    }
    
    if (!allSuccessful) {
        qWarning() << "Failed to initialize services:" << failedServices;
    }
    
    return allSuccessful;
}

bool ServiceRegistry::initializeService(const QString& serviceId) {
    QMutexLocker locker(&m_mutex);
    
    auto it = m_services.find(serviceId);
    if (it == m_services.end()) {
        qWarning() << "Service not found for initialization:" << serviceId;
        return false;
    }
    
    ServiceInterface* service = it.value();
    if (service->isInitialized()) {
        // Service bereits initialisiert
        return true;
    }
    
    // Überprüfe Abhängigkeiten
    QList<QString> dependencies = getDependenciesForService(serviceId);
    for (const QString& depId : dependencies) {
        if (!hasService(depId)) {
            qWarning() << "Missing dependency" << depId << "for service" << serviceId;
            return false;
        }
        
        // Initialisiere Abhängigkeit falls nötig
        auto depIt = m_services.find(depId);
        ServiceInterface* depService = depIt.value();
        if (!depService->isInitialized() && !initializeService(depId)) {
            qWarning() << "Failed to initialize dependency" << depId << "for service" << serviceId;
            return false;
        }
    }
    
    // Initialisiere den Service
    qDebug() << "Initializing service:" << serviceId;
    bool success = service->initialize();
    
    if (!success) {
        qWarning() << "Failed to initialize service:" << serviceId;
    }
    
    return success;
}

bool ServiceRegistry::shutdownService(const QString& serviceId) {
    QMutexLocker locker(&m_mutex);
    
    auto it = m_services.find(serviceId);
    if (it == m_services.end()) {
        qWarning() << "Service not found for shutdown:" << serviceId;
        return false;
    }
    
    ServiceInterface* service = it.value();
    if (!service->isInitialized()) {
        // Service nicht initialisiert, nichts zu tun
        return true;
    }
    
    // Überprüfe, ob andere Services von diesem abhängen
    QList<QString> dependentServices = getServicesDependentOn(serviceId);
    
    // Shutdown abhängige Services zuerst
    for (const QString& depId : dependentServices) {
        if (!shutdownService(depId)) {
            qWarning() << "Failed to shutdown dependent service" << depId << "for service" << serviceId;
        }
    }
    
    // Shutdown den Service
    qDebug() << "Shutting down service:" << serviceId;
    service->shutdown();
    
    return true;
}

void ServiceRegistry::shutdownAllServices() {
    QMutexLocker locker(&m_mutex);
    
    // Shutdown in umgekehrter Initialisierungsreihenfolge
    QList<QString> initOrder = getInitializationOrder();
    
    // Umkehren für Shutdown
    for (int i = initOrder.size() - 1; i >= 0; i--) {
        shutdownService(initOrder[i]);
    }
}

QList<QString> ServiceRegistry::getInitializationOrder() {
    // Erstelle einen DAG für die topologische Sortierung
    QMap<QString, QList<QString>> graph;
    QMap<QString, int> inDegree;
    
    // Initialisiere den Graphen
    for (auto it = m_services.begin(); it != m_services.end(); ++it) {
        QString serviceId = it.key();
        graph[serviceId] = QList<QString>();
        inDegree[serviceId] = 0;
    }
    
    // Erstelle Kanten im Graphen
    for (auto it = m_services.begin(); it != m_services.end(); ++it) {
        QString serviceId = it.key();
        QList<QString> dependencies = getDependenciesForService(serviceId);
        
        for (const QString& depId : dependencies) {
            if (graph.contains(depId)) {
                graph[depId].append(serviceId);
                inDegree[serviceId]++;
            }
        }
    }
    
    // Topologische Sortierung mit Kahn's Algorithmus
    QList<QString> result;
    QQueue<QString> queue;
    
    // Füge alle Knoten mit inDegree 0 zur Queue hinzu
    for (auto it = inDegree.begin(); it != inDegree.end(); ++it) {
        if (it.value() == 0) {
            queue.enqueue(it.key());
        }
    }
    
    while (!queue.isEmpty()) {
        QString current = queue.dequeue();
        result.append(current);
        
        for (const QString& dependent : graph[current]) {
            inDegree[dependent]--;
            if (inDegree[dependent] == 0) {
                queue.enqueue(dependent);
            }
        }
    }
    
    // Prüfe auf zyklische Abhängigkeiten
    if (result.size() != m_services.size()) {
        qWarning() << "Cyclic dependency detected in service initialization order";
        
        // Fallback: Einfache Auflistung aller Services
        result.clear();
        for (auto it = m_services.begin(); it != m_services.end(); ++it) {
            result.append(it.key());
        }
    }
    
    return result;
}

QList<QString> ServiceRegistry::getDependenciesForService(const QString& serviceId) {
    // In einer echten Implementierung würden wir die Abhängigkeiten vom Service selbst abfragen
    // Dies ist eine Beispielimplementierung
    auto it = m_services.find(serviceId);
    if (it == m_services.end()) {
        return QList<QString>();
    }
    
    // Hier müsste eine Art getDependencies() Methode im ServiceInterface definiert sein
    // Da sie nicht in der ursprünglichen Schnittstelle existiert, geben wir eine leere Liste zurück
    return QList<QString>();
}

QList<QString> ServiceRegistry::getServicesDependentOn(const QString& serviceId) {
    QList<QString> dependentServices;
    
    // Durchlaufe alle Services und prüfe, ob sie von diesem Service abhängen
    for (auto it = m_services.begin(); it != m_services.end(); ++it) {
        QString currentId = it.key();
        QList<QString> dependencies = getDependenciesForService(currentId);
        
        if (dependencies.contains(serviceId)) {
            dependentServices.append(currentId);
        }
    }
    
    return dependentServices;
}

} // namespace Core
} // namespace VivoX