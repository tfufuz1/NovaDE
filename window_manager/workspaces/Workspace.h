#pragma once

#include <QObject>
#include <QString>
#include <QRect>

namespace VivoX::WindowManager {

/**
 * @brief The Workspace class represents a virtual desktop workspace.
 * 
 * It contains a collection of windows and provides properties for
 * workspace identification and management.
 */
class Workspace : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString id READ id WRITE setId NOTIFY idChanged)
    Q_PROPERTY(QString name READ name WRITE setName NOTIFY nameChanged)
    Q_PROPERTY(QRect geometry READ geometry WRITE setGeometry NOTIFY geometryChanged)
    Q_PROPERTY(bool active READ isActive WRITE setActive NOTIFY activeChanged)

public:
    explicit Workspace(QObject *parent = nullptr);
    ~Workspace();

    /**
     * @brief Get the workspace ID
     * @return The workspace ID
     */
    QString id() const;

    /**
     * @brief Set the workspace ID
     * @param id The workspace ID
     */
    void setId(const QString &id);

    /**
     * @brief Get the workspace name
     * @return The workspace name
     */
    QString name() const;

    /**
     * @brief Set the workspace name
     * @param name The workspace name
     */
    void setName(const QString &name);

    /**
     * @brief Get the workspace geometry
     * @return The workspace geometry
     */
    QRect geometry() const;

    /**
     * @brief Set the workspace geometry
     * @param geometry The workspace geometry
     */
    void setGeometry(const QRect &geometry);

    /**
     * @brief Check if the workspace is active
     * @return True if the workspace is active
     */
    bool isActive() const;

    /**
     * @brief Set the workspace active state
     * @param active The active state
     */
    void setActive(bool active);

signals:
    /**
     * @brief Signal emitted when the workspace ID changes
     * @param id The new ID
     */
    void idChanged(const QString &id);

    /**
     * @brief Signal emitted when the workspace name changes
     * @param name The new name
     */
    void nameChanged(const QString &name);

    /**
     * @brief Signal emitted when the workspace geometry changes
     * @param geometry The new geometry
     */
    void geometryChanged(const QRect &geometry);

    /**
     * @brief Signal emitted when the workspace active state changes
     * @param active The new active state
     */
    void activeChanged(bool active);

private:
    // Workspace ID
    QString m_id;
    
    // Workspace name
    QString m_name;
    
    // Workspace geometry
    QRect m_geometry;
    
    // Whether the workspace is active
    bool m_active;
};

} // namespace VivoX::WindowManager
