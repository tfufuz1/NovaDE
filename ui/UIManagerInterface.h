#pragma once

#include <QObject>
#include <QString>
#include <QList>
#include <QVariant>

namespace VivoX {
namespace UI {

class Widget;
class Panel;

/**
 * @brief Interface for the UI manager
 * 
 * This interface defines the functionality that the UI manager must implement.
 * It is responsible for managing the user interface, including panels, widgets, and themes.
 */
class UIManagerInterface {
public:
    virtual ~UIManagerInterface() = default;

    /**
     * @brief Initialize the UI manager
     * 
     * @return bool True if initialization was successful, false otherwise
     */
    virtual bool initialize() = 0;

    /**
     * @brief Get the main QML engine
     * 
     * @return QObject* The QML engine
     */
    virtual QObject* qmlEngine() const = 0;

    /**
     * @brief Load a QML component
     * 
     * @param url The URL of the QML component to load
     * @return QObject* The loaded component
     */
    virtual QObject* loadQmlComponent(const QString& url) = 0;

    /**
     * @brief Register a panel with the UI manager
     * 
     * @param panel The panel to register
     */
    virtual void registerPanel(Panel* panel) = 0;

    /**
     * @brief Unregister a panel from the UI manager
     * 
     * @param panel The panel to unregister
     */
    virtual void unregisterPanel(Panel* panel) = 0;

    /**
     * @brief Get all registered panels
     * 
     * @return QList<Panel*> The list of panels
     */
    virtual QList<Panel*> panels() const = 0;

    /**
     * @brief Get a panel by ID
     * 
     * @param id The panel ID
     * @return Panel* The panel, or nullptr if not found
     */
    virtual Panel* panelById(const QString& id) const = 0;

    /**
     * @brief Register a widget with the UI manager
     * 
     * @param widget The widget to register
     */
    virtual void registerWidget(Widget* widget) = 0;

    /**
     * @brief Unregister a widget from the UI manager
     * 
     * @param widget The widget to unregister
     */
    virtual void unregisterWidget(Widget* widget) = 0;

    /**
     * @brief Get all registered widgets
     * 
     * @return QList<Widget*> The list of widgets
     */
    virtual QList<Widget*> widgets() const = 0;

    /**
     * @brief Get a widget by ID
     * 
     * @param id The widget ID
     * @return Widget* The widget, or nullptr if not found
     */
    virtual Widget* widgetById(const QString& id) const = 0;

    /**
     * @brief Set the current theme
     * 
     * @param themeName The name of the theme to set
     */
    virtual void setTheme(const QString& themeName) = 0;

    /**
     * @brief Get the current theme name
     * 
     * @return QString The current theme name
     */
    virtual QString currentTheme() const = 0;

    /**
     * @brief Get a list of available themes
     * 
     * @return QStringList The list of available themes
     */
    virtual QStringList availableThemes() const = 0;

    /**
     * @brief Get a theme property
     * 
     * @param property The property name
     * @return QVariant The property value
     */
    virtual QVariant themeProperty(const QString& property) const = 0;

    /**
     * @brief Show the dashboard
     */
    virtual void showDashboard() = 0;

    /**
     * @brief Hide the dashboard
     */
    virtual void hideDashboard() = 0;

    /**
     * @brief Show the overview
     */
    virtual void showOverview() = 0;

    /**
     * @brief Hide the overview
     */
    virtual void hideOverview() = 0;

    /**
     * @brief Show the command palette
     */
    virtual void showCommandPalette() = 0;

    /**
     * @brief Hide the command palette
     */
    virtual void hideCommandPalette() = 0;
};

} // namespace UI
} // namespace VivoX
