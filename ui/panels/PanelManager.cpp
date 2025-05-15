#include "PanelManager.h"
#include <QDebug>
#include <QUuid>
#include "../core/ConfigManager.h"
#include "../core/Logger.h"

namespace VivoX {
namespace Panel {

PanelManager::PanelManager() : QObject(nullptr), m_nextPanelId(1)
{
    qDebug() << "PanelManager initialized";
}

PanelManager::~PanelManager()
{
    // Entferne alle Panels
    QStringList panelIds = m_panels.keys();
    for (const QString& panelId : panelIds) {
        removePanel(panelId);
    }
    
    qDebug() << "PanelManager destroyed";
}

PanelManager& PanelManager::instance()
{
    static PanelManager instance;
    return instance;
}

void PanelManager::initialize()
{
    // Lade die Panel-Konfiguration
    loadPanelConfiguration();
    
    qDebug() << "PanelManager initialized";
}

QString PanelManager::createPanel(const QString& panelType, PanelPosition position, const QString& name)
{
    if (!m_panelFactories.contains(panelType)) {
        Core::Logger::instance().error("Panel type not registered: " + panelType, "PanelManager");
        return QString();
    }
    
    // Erstelle das Panel mit der Factory-Funktion
    PanelInterface* panel = m_panelFactories[panelType]();
    if (!panel) {
        Core::Logger::instance().error("Failed to create panel of type: " + panelType, "PanelManager");
        return QString();
    }
    
    // Generiere eine eindeutige ID
    QString panelId = generatePanelId();
    
    // Setze die Eigenschaften des Panels
    panel->setPosition(position);
    if (!name.isEmpty()) {
        panel->setPanelName(name);
    }
    
    // Verbinde Signale
    setupPanelConnections(panel);
    
    // Speichere das Panel
    m_panels[panelId] = panel;
    
    emit panelCreated(panelId);
    Core::Logger::instance().info("Panel created: " + panelId + " of type: " + panelType, "PanelManager");
    
    return panelId;
}

bool PanelManager::removePanel(const QString& panelId)
{
    if (!m_panels.contains(panelId)) {
        Core::Logger::instance().warning("Panel not found: " + panelId, "PanelManager");
        return false;
    }
    
    PanelInterface* panel = m_panels[panelId];
    
    // Trenne Signale
    disconnect(panel, nullptr, this, nullptr);
    
    // Entferne das Panel
    m_panels.remove(panelId);
    
    // Lösche das Panel-Objekt
    delete panel;
    
    emit panelRemoved(panelId);
    Core::Logger::instance().info("Panel removed: " + panelId, "PanelManager");
    
    return true;
}

PanelInterface* PanelManager::getPanel(const QString& panelId) const
{
    return m_panels.value(panelId, nullptr);
}

QMap<QString, PanelInterface*> PanelManager::getPanels() const
{
    return m_panels;
}

QStringList PanelManager::getPanelIds() const
{
    return m_panels.keys();
}

QStringList PanelManager::getPanelsByPosition(PanelPosition position) const
{
    QStringList result;
    
    for (auto it = m_panels.begin(); it != m_panels.end(); ++it) {
        if (it.value()->position() == position) {
            result << it.key();
        }
    }
    
    return result;
}

bool PanelManager::registerPanelType(const QString& panelType, std::function<PanelInterface*()> factory)
{
    if (m_panelFactories.contains(panelType)) {
        Core::Logger::instance().warning("Panel type already registered: " + panelType, "PanelManager");
        return false;
    }
    
    m_panelFactories[panelType] = factory;
    Core::Logger::instance().info("Panel type registered: " + panelType, "PanelManager");
    
    return true;
}

QStringList PanelManager::getAvailablePanelTypes() const
{
    return m_panelFactories.keys();
}

bool PanelManager::savePanelConfiguration()
{
    Core::ConfigManager& configManager = Core::ConfigManager::instance();
    
    // Speichere die Anzahl der Panels
    configManager.setValue("panels/count", m_panels.size());
    
    // Speichere die Panel-Typen
    int index = 0;
    for (auto it = m_panelFactories.begin(); it != m_panelFactories.end(); ++it) {
        configManager.setValue("panels/types/" + QString::number(index) + "/name", it.key());
        index++;
    }
    configManager.setValue("panels/types/count", index);
    
    // Speichere die Panel-Konfigurationen
    index = 0;
    for (auto it = m_panels.begin(); it != m_panels.end(); ++it) {
        QString panelId = it.key();
        PanelInterface* panel = it.value();
        
        configManager.setValue("panels/panel" + QString::number(index) + "/id", panelId);
        configManager.setValue("panels/panel" + QString::number(index) + "/name", panel->panelName());
        configManager.setValue("panels/panel" + QString::number(index) + "/position", static_cast<int>(panel->position()));
        configManager.setValue("panels/panel" + QString::number(index) + "/geometry", panel->geometry());
        configManager.setValue("panels/panel" + QString::number(index) + "/visible", panel->isVisible());
        configManager.setValue("panels/panel" + QString::number(index) + "/locked", panel->isLocked());
        configManager.setValue("panels/panel" + QString::number(index) + "/autoHide", panel->autoHide());
        
        // Speichere die Widgets des Panels
        QStringList widgets = panel->widgets();
        configManager.setValue("panels/panel" + QString::number(index) + "/widgets/count", widgets.size());
        for (int i = 0; i < widgets.size(); ++i) {
            configManager.setValue("panels/panel" + QString::number(index) + "/widgets/" + QString::number(i), widgets[i]);
        }
        
        index++;
    }
    
    Core::Logger::instance().info("Panel configuration saved", "PanelManager");
    return true;
}

bool PanelManager::loadPanelConfiguration()
{
    Core::ConfigManager& configManager = Core::ConfigManager::instance();
    
    // Entferne alle bestehenden Panels
    QStringList panelIds = m_panels.keys();
    for (const QString& panelId : panelIds) {
        removePanel(panelId);
    }
    
    // Lade die Panel-Konfigurationen
    int panelCount = configManager.getValue("panels/count", 0).toInt();
    for (int i = 0; i < panelCount; ++i) {
        QString panelId = configManager.getValue("panels/panel" + QString::number(i) + "/id").toString();
        QString panelName = configManager.getValue("panels/panel" + QString::number(i) + "/name").toString();
        PanelPosition position = static_cast<PanelPosition>(configManager.getValue("panels/panel" + QString::number(i) + "/position", 0).toInt());
        QRect geometry = configManager.getValue("panels/panel" + QString::number(i) + "/geometry").toRect();
        bool visible = configManager.getValue("panels/panel" + QString::number(i) + "/visible", true).toBool();
        bool locked = configManager.getValue("panels/panel" + QString::number(i) + "/locked", false).toBool();
        bool autoHide = configManager.getValue("panels/panel" + QString::number(i) + "/autoHide", false).toBool();
        
        // Erstelle das Panel
        // Hier müssten wir den Panel-Typ kennen, was in dieser einfachen Implementierung nicht der Fall ist
        // In einer vollständigen Implementierung würde der Typ mit gespeichert werden
        if (!m_panelFactories.isEmpty()) {
            QString panelType = m_panelFactories.keys().first();
            QString newPanelId = createPanel(panelType, position, panelName);
            
            if (!newPanelId.isEmpty()) {
                PanelInterface* panel = getPanel(newPanelId);
                if (panel) {
                    panel->setGeometry(geometry);
                    panel->setVisible(visible);
                    panel->setLocked(locked);
                    panel->setAutoHide(autoHide);
                    
                    // Lade die Widgets des Panels
                    int widgetCount = configManager.getValue("panels/panel" + QString::number(i) + "/widgets/count", 0).toInt();
                    for (int j = 0; j < widgetCount; ++j) {
                        QString widgetId = configManager.getValue("panels/panel" + QString::number(i) + "/widgets/" + QString::number(j)).toString();
                        panel->addWidget(widgetId);
                    }
                }
            }
        }
    }
    
    Core::Logger::instance().info("Panel configuration loaded", "PanelManager");
    return true;
}

bool PanelManager::resetPanelConfiguration()
{
    // Entferne alle bestehenden Panels
    QStringList panelIds = m_panels.keys();
    for (const QString& panelId : panelIds) {
        removePanel(panelId);
    }
    
    // Erstelle Standardpanels
    if (!m_panelFactories.isEmpty()) {
        QString panelType = m_panelFactories.keys().first();
        
        // Erstelle ein Panel am unteren Bildschirmrand
        createPanel(panelType, PanelPosition::Bottom, "Bottom Panel");
    }
    
    Core::Logger::instance().info("Panel configuration reset to defaults", "PanelManager");
    return true;
}

QString PanelManager::generatePanelId() const
{
    return "panel_" + QUuid::createUuid().toString(QUuid::WithoutBraces).left(8);
}

void PanelManager::setupPanelConnections(PanelInterface* panel)
{
    if (!panel) {
        return;
    }
    
    connect(panel, &PanelInterface::positionChanged, [this, panel](PanelPosition position) {
        QString panelId = panel->panelId();
        emit panelPositionChanged(panelId, position);
    });
    
    connect(panel, &PanelInterface::geometryChanged, [this, panel](const QRect& geometry) {
        QString panelId = panel->panelId();
        emit panelGeometryChanged(panelId, geometry);
    });
    
    connect(panel, &PanelInterface::visibleChanged, [this, panel](bool visible) {
        QString panelId = panel->panelId();
        emit panelVisibilityChanged(panelId, visible);
    });
}

} // namespace Panel
} // namespace VivoX
