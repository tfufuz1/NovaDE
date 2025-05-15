#include "GestureEngine.h"
#include "GestureRecognizer.h"

#include <QDebug>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include <QFile>
#include <QVariant>
#include <QVector2D>
#include <cmath>

namespace VivoX::Input {

GestureEngine::GestureEngine(QObject *parent)
    : QObject(parent)
{
    qDebug() << "GestureEngine created";
}

GestureEngine::~GestureEngine()
{
    // Clean up recognizers
    for (GestureRecognizer *recognizer : m_recognizers) {
        delete recognizer;
    }
    
    qDebug() << "GestureEngine destroyed";
}

bool GestureEngine::initialize()
{
    // Initialize recognizers
    initializeRecognizers();
    
    // Load gestures from configuration
    loadGestures();
    
    qDebug() << "GestureEngine initialized";
    return true;
}

bool GestureEngine::handlePointerButton(const QPoint &position, quint32 button, bool pressed)
{
    bool handled = false;
    
    // Pass to all recognizers
    for (GestureRecognizer *recognizer : m_recognizers) {
        if (recognizer->handlePointerButton(position, button, pressed)) {
            handled = true;
        }
    }
    
    return handled;
}

bool GestureEngine::handlePointerMotion(const QPoint &position)
{
    bool handled = false;
    
    // Pass to all recognizers
    for (GestureRecognizer *recognizer : m_recognizers) {
        if (recognizer->handlePointerMotion(position)) {
            handled = true;
        }
    }
    
    return handled;
}

bool GestureEngine::handleTouchPoint(qint32 id, const QPoint &position, bool pressed)
{
    // Update touch points
    if (pressed) {
        m_touchPoints[id] = position;
    } else {
        m_touchPoints.remove(id);
    }
    
    bool handled = false;
    
    // Pass to all recognizers
    for (GestureRecognizer *recognizer : m_recognizers) {
        if (recognizer->handleTouchPoint(id, position, pressed)) {
            handled = true;
        }
    }
    
    return handled;
}

bool GestureEngine::handleTouchMotion(qint32 id, const QPoint &position)
{
    // Update touch point
    if (m_touchPoints.contains(id)) {
        m_touchPoints[id] = position;
    }
    
    bool handled = false;
    
    // Pass to all recognizers
    for (GestureRecognizer *recognizer : m_recognizers) {
        if (recognizer->handleTouchMotion(id, position)) {
            handled = true;
        }
    }
    
    return handled;
}

bool GestureEngine::handleTouchFrame()
{
    bool handled = false;
    
    // Pass to all recognizers
    for (GestureRecognizer *recognizer : m_recognizers) {
        if (recognizer->handleTouchFrame()) {
            handled = true;
        }
    }
    
    return handled;
}

bool GestureEngine::handleScroll(qreal delta, quint32 orientation)
{
    bool handled = false;
    
    // Pass to all recognizers
    for (GestureRecognizer *recognizer : m_recognizers) {
        if (recognizer->handleScroll(delta, orientation)) {
            handled = true;
        }
    }
    
    return handled;
}

bool GestureEngine::registerGesture(GestureType type, const QVariantMap &parameters, const QString &actionId)
{
    if (actionId.isEmpty()) {
        qWarning() << "Cannot register gesture with empty action ID";
        return false;
    }
    
    // Create gesture config
    GestureConfig config;
    config.type = type;
    config.parameters = parameters;
    config.actionId = actionId;
    
    // Add to list
    m_gestureConfigs.append(config);
    
    qDebug() << "Registered gesture of type" << type << "with action" << actionId;
    
    return true;
}

bool GestureEngine::unregisterGesture(GestureType type, const QVariantMap &parameters)
{
    for (int i = 0; i < m_gestureConfigs.size(); ++i) {
        const GestureConfig &config = m_gestureConfigs[i];
        
        if (config.type == type) {
            // Check if parameters match
            bool match = true;
            for (auto it = parameters.begin(); it != parameters.end(); ++it) {
                if (!config.parameters.contains(it.key()) || config.parameters[it.key()] != it.value()) {
                    match = false;
                    break;
                }
            }
            
            if (match) {
                m_gestureConfigs.remove(i);
                qDebug() << "Unregistered gesture of type" << type;
                return true;
            }
        }
    }
    
    qWarning() << "Gesture not found for unregistration";
    return false;
}

bool GestureEngine::loadGestures()
{
    // Clear existing gestures
    m_gestureConfigs.clear();
    
    // Load from configuration file
    QFile file(":/config/gestures.json");
    
    if (!file.open(QIODevice::ReadOnly)) {
        qWarning() << "Failed to open gestures configuration file";
        
        // Load default gestures
        registerGesture(Swipe, {{"direction", "right"}, {"fingers", 3}}, "workspace.next");
        registerGesture(Swipe, {{"direction", "left"}, {"fingers", 3}}, "workspace.previous");
        registerGesture(Swipe, {{"direction", "up"}, {"fingers", 3}}, "window.maximize");
        registerGesture(Swipe, {{"direction", "down"}, {"fingers", 3}}, "window.minimize");
        registerGesture(Pinch, {{"direction", "in"}, {"fingers", 2}}, "window.scale.decrease");
        registerGesture(Pinch, {{"direction", "out"}, {"fingers", 2}}, "window.scale.increase");
        registerGesture(EdgeSwipe, {{"edge", "top"}}, "overview.show");
        registerGesture(EdgeSwipe, {{"edge", "bottom"}}, "dock.show");
        registerGesture(EdgeSwipe, {{"edge", "left"}}, "sidebar.show");
        registerGesture(EdgeSwipe, {{"edge", "right"}}, "notifications.show");
        
        return true;
    }
    
    QByteArray data = file.readAll();
    file.close();
    
    QJsonDocument doc = QJsonDocument::fromJson(data);
    
    if (doc.isNull() || !doc.isObject()) {
        qWarning() << "Invalid gestures configuration format";
        return false;
    }
    
    QJsonObject obj = doc.object();
    QJsonArray gestures = obj["gestures"].toArray();
    
    for (int i = 0; i < gestures.size(); ++i) {
        QJsonObject gestureObj = gestures[i].toObject();
        int typeInt = gestureObj["type"].toInt();
        QJsonObject paramsObj = gestureObj["parameters"].toObject();
        QString actionId = gestureObj["action"].toString();
        
        if (typeInt >= 0 && typeInt <= RightButtonScroll && !actionId.isEmpty()) {
            GestureType type = static_cast<GestureType>(typeInt);
            
            // Convert JSON parameters to QVariantMap
            QVariantMap parameters;
            for (auto it = paramsObj.begin(); it != paramsObj.end(); ++it) {
                parameters[it.key()] = it.value().toVariant();
            }
            
            registerGesture(type, parameters, actionId);
        }
    }
    
    qDebug() << "Loaded" << m_gestureConfigs.size() << "gestures from configuration";
    
    return true;
}

bool GestureEngine::saveGestures()
{
    QJsonObject obj;
    QJsonArray gestures;
    
    for (const GestureConfig &config : m_gestureConfigs) {
        QJsonObject gestureObj;
        gestureObj["type"] = static_cast<int>(config.type);
        
        // Convert QVariantMap parameters to JSON
        QJsonObject paramsObj;
        for (auto it = config.parameters.begin(); it != config.parameters.end(); ++it) {
            paramsObj[it.key()] = QJsonValue::fromVariant(it.value());
        }
        
        gestureObj["parameters"] = paramsObj;
        gestureObj["action"] = config.actionId;
        
        gestures.append(gestureObj);
    }
    
    obj["gestures"] = gestures;
    
    QJsonDocument doc(obj);
    
    QFile file(":/config/gestures.json");
    
    if (!file.open(QIODevice::WriteOnly)) {
        qWarning() << "Failed to open gestures configuration file for writing";
        return false;
    }
    
    file.write(doc.toJson());
    file.close();
    
    qDebug() << "Saved" << m_gestureConfigs.size() << "gestures to configuration";
    
    return true;
}

void GestureEngine::initializeRecognizers()
{
    // Create and add recognizers for each gesture type
    
    // Connect signals from recognizers
    for (GestureRecognizer *recognizer : m_recognizers) {
        connect(recognizer, &GestureRecognizer::gestureRecognized, this, 
            [this](GestureType type, const QVariantMap &parameters) {
                processGesture(type, parameters);
            });
            
        connect(recognizer, &GestureRecognizer::gestureFeedback, this, &GestureEngine::gestureFeedback);
    }
    
    qDebug() << "Initialized" << m_recognizers.size() << "gesture recognizers";
}

void GestureEngine::processGesture(GestureType type, const QVariantMap &parameters)
{
    // Find matching gesture config
    for (const GestureConfig &config : m_gestureConfigs) {
        if (config.type == type) {
            // Check if parameters match
            bool match = true;
            for (auto it = config.parameters.begin(); it != config.parameters.end(); ++it) {
                if (!parameters.contains(it.key()) || parameters[it.key()] != it.value()) {
                    match = false;
                    break;
                }
            }
            
            if (match) {
                qDebug() << "Gesture recognized:" << type << "Action:" << config.actionId;
                
                // Emit signal
                emit gestureRecognized(type, parameters);
                
                // TODO: Trigger action via ActionManager
                
                return;
            }
        }
    }
    
    qDebug() << "Gesture recognized but no matching action found:" << type;
}

} // namespace VivoX::Input
