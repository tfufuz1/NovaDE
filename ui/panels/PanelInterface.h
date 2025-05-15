#pragma once

#include <QObject>
#include <QString>
#include <QVariant>
#include <QRect>
#include <QPoint>
#include <QSize>
#include <QVector>
#include <QMap>

namespace VivoX {
namespace Panel {

/**
 * @brief Aufzählung der verschiedenen Panel-Positionen
 */
enum class PanelPosition {
    Top,
    Bottom,
    Left,
    Right,
    Floating
};

/**
 * @brief Die PanelInterface-Klasse definiert die Schnittstelle für Panels im VivoX-System.
 * 
 * Diese Klasse muss von allen Panel-Implementierungen implementiert werden, um mit dem
 * VivoX-Panel-Framework zu interagieren. Sie definiert die Eigenschaften und Methoden eines Panels.
 */
class PanelInterface : public QObject
{
    Q_OBJECT
    Q_PROPERTY(QString panelId READ panelId CONSTANT)
    Q_PROPERTY(QString panelName READ panelName WRITE setPanelName NOTIFY panelNameChanged)
    Q_PROPERTY(PanelPosition position READ position WRITE setPosition NOTIFY positionChanged)
    Q_PROPERTY(QRect geometry READ geometry WRITE setGeometry NOTIFY geometryChanged)
    Q_PROPERTY(bool visible READ isVisible WRITE setVisible NOTIFY visibleChanged)
    Q_PROPERTY(bool locked READ isLocked WRITE setLocked NOTIFY lockedChanged)
    Q_PROPERTY(bool autoHide READ autoHide WRITE setAutoHide NOTIFY autoHideChanged)

public:
    /**
     * @brief Konstruktor für das Panel-Interface.
     * @param parent Elternobjekt
     */
    explicit PanelInterface(QObject* parent = nullptr) : QObject(parent) {}
    
    /**
     * @brief Virtueller Destruktor.
     */
    virtual ~PanelInterface() {}

    /**
     * @brief Gibt die ID des Panels zurück.
     * @return Panel-ID
     */
    virtual QString panelId() const = 0;

    /**
     * @brief Gibt den Namen des Panels zurück.
     * @return Panel-Name
     */
    virtual QString panelName() const = 0;

    /**
     * @brief Setzt den Namen des Panels.
     * @param name Neuer Name
     */
    virtual void setPanelName(const QString& name) = 0;

    /**
     * @brief Gibt die Position des Panels zurück.
     * @return Panel-Position
     */
    virtual PanelPosition position() const = 0;

    /**
     * @brief Setzt die Position des Panels.
     * @param position Neue Position
     */
    virtual void setPosition(PanelPosition position) = 0;

    /**
     * @brief Gibt die Geometrie des Panels zurück.
     * @return Panel-Geometrie
     */
    virtual QRect geometry() const = 0;

    /**
     * @brief Setzt die Geometrie des Panels.
     * @param geometry Neue Geometrie
     */
    virtual void setGeometry(const QRect& geometry) = 0;

    /**
     * @brief Gibt an, ob das Panel sichtbar ist.
     * @return true wenn sichtbar, false wenn unsichtbar
     */
    virtual bool isVisible() const = 0;

    /**
     * @brief Setzt die Sichtbarkeit des Panels.
     * @param visible Neue Sichtbarkeit
     */
    virtual void setVisible(bool visible) = 0;

    /**
     * @brief Gibt an, ob das Panel gesperrt ist.
     * @return true wenn gesperrt, false wenn entsperrt
     */
    virtual bool isLocked() const = 0;

    /**
     * @brief Sperrt oder entsperrt das Panel.
     * @param locked Neuer Sperrstatus
     */
    virtual void setLocked(bool locked) = 0;

    /**
     * @brief Gibt an, ob das Panel automatisch ausgeblendet wird.
     * @return true wenn Auto-Hide aktiviert ist, false sonst
     */
    virtual bool autoHide() const = 0;

    /**
     * @brief Aktiviert oder deaktiviert das automatische Ausblenden des Panels.
     * @param autoHide Neuer Auto-Hide-Status
     */
    virtual void setAutoHide(bool autoHide) = 0;

    /**
     * @brief Fügt ein Widget zum Panel hinzu.
     * @param widgetId ID des Widgets
     * @param position Position im Panel (optional, -1 für Anhängen am Ende)
     * @return true wenn erfolgreich, false bei Fehler
     */
    virtual bool addWidget(const QString& widgetId, int position = -1) = 0;

    /**
     * @brief Entfernt ein Widget aus dem Panel.
     * @param widgetId ID des Widgets
     * @return true wenn erfolgreich, false bei Fehler
     */
    virtual bool removeWidget(const QString& widgetId) = 0;

    /**
     * @brief Gibt die IDs aller Widgets im Panel zurück.
     * @return Liste der Widget-IDs
     */
    virtual QStringList widgets() const = 0;

    /**
     * @brief Ändert die Position eines Widgets im Panel.
     * @param widgetId ID des Widgets
     * @param newPosition Neue Position
     * @return true wenn erfolgreich, false bei Fehler
     */
    virtual bool moveWidget(const QString& widgetId, int newPosition) = 0;

    /**
     * @brief Zeigt das Panel an.
     */
    virtual void show() = 0;

    /**
     * @brief Blendet das Panel aus.
     */
    virtual void hide() = 0;

    /**
     * @brief Aktualisiert das Panel.
     */
    virtual void update() = 0;

signals:
    /**
     * @brief Signal, das ausgelöst wird, wenn sich der Name des Panels ändert.
     * @param name Neuer Name
     */
    void panelNameChanged(const QString& name);

    /**
     * @brief Signal, das ausgelöst wird, wenn sich die Position des Panels ändert.
     * @param position Neue Position
     */
    void positionChanged(PanelPosition position);

    /**
     * @brief Signal, das ausgelöst wird, wenn sich die Geometrie des Panels ändert.
     * @param geometry Neue Geometrie
     */
    void geometryChanged(const QRect& geometry);

    /**
     * @brief Signal, das ausgelöst wird, wenn sich die Sichtbarkeit des Panels ändert.
     * @param visible Neue Sichtbarkeit
     */
    void visibleChanged(bool visible);

    /**
     * @brief Signal, das ausgelöst wird, wenn sich der Sperrstatus des Panels ändert.
     * @param locked Neuer Sperrstatus
     */
    void lockedChanged(bool locked);

    /**
     * @brief Signal, das ausgelöst wird, wenn sich der Auto-Hide-Status des Panels ändert.
     * @param autoHide Neuer Auto-Hide-Status
     */
    void autoHideChanged(bool autoHide);

    /**
     * @brief Signal, das ausgelöst wird, wenn ein Widget zum Panel hinzugefügt wird.
     * @param widgetId ID des hinzugefügten Widgets
     * @param position Position im Panel
     */
    void widgetAdded(const QString& widgetId, int position);

    /**
     * @brief Signal, das ausgelöst wird, wenn ein Widget aus dem Panel entfernt wird.
     * @param widgetId ID des entfernten Widgets
     */
    void widgetRemoved(const QString& widgetId);

    /**
     * @brief Signal, das ausgelöst wird, wenn ein Widget im Panel verschoben wird.
     * @param widgetId ID des verschobenen Widgets
     * @param newPosition Neue Position
     */
    void widgetMoved(const QString& widgetId, int newPosition);
};

} // namespace Panel
} // namespace VivoX
