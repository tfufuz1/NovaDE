#pragma once

#include <QObject>
#include <QString>
#include <QQmlEngine>
#include <QJSEngine>
#include <QMap>
#include <QVariant>
#include <QColor>
#include <QFont>
#include <QUrl>
#include <memory>
#include <functional>

namespace VivoX::UI {

class Panel;
class Widget;
class Theme;
class WidgetFactory;

/**
 * @brief The UIManager class manages the UI components and their integration.
 * 
 * It is responsible for registering QML types, loading QML files, and
 * providing a bridge between the C++ backend and the QML frontend.
 * 
 * Features:
 * - QML integration and type registration
 * - Theme management
 * - Panel management
 * - Widget management
 * - Layout management
 * - Responsive design support
 * - Accessibility features
 * - Internationalization support
 */
class UIManager : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString currentTheme READ currentTheme WRITE setCurrentTheme NOTIFY currentThemeChanged)
    Q_PROPERTY(bool darkMode READ isDarkMode WRITE setDarkMode NOTIFY darkModeChanged)
    Q_PROPERTY(QColor accentColor READ accentColor WRITE setAccentColor NOTIFY accentColorChanged)
    Q_PROPERTY(QFont systemFont READ systemFont WRITE setSystemFont NOTIFY systemFontChanged)
    Q_PROPERTY(int uiScale READ uiScale WRITE setUiScale NOTIFY uiScaleChanged)
    Q_PROPERTY(bool accessibilityMode READ accessibilityMode WRITE setAccessibilityMode NOTIFY accessibilityModeChanged)
    Q_PROPERTY(QString language READ language WRITE setLanguage NOTIFY languageChanged)

public:
    explicit UIManager(QObject *parent = nullptr);
    ~UIManager();

    /**
     * @brief Initialize the UI manager
     * @return True if initialization was successful
     */
    bool initialize();
    
    /**
     * @brief Shutdown the UI manager
     */
    void shutdown();

    /**
     * @brief Get the QML engine
     * @return The QML engine
     */
    QQmlEngine *qmlEngine() const;

    /**
     * @brief Register a QML type
     * @param uri The URI to register the type under
     * @param major The major version
     * @param minor The minor version
     * @param qmlName The name to use in QML
     * @return True if registration was successful
     */
    template<typename T>
    bool registerQmlType(const char *uri, int major, int minor, const char *qmlName) {
        qmlRegisterType<T>(uri, major, minor, qmlName);
        return true;
    }

    /**
     * @brief Register a singleton QML type
     * @param uri The URI to register the type under
     * @param major The major version
     * @param minor The minor version
     * @param qmlName The name to use in QML
     * @param callback The callback to create the singleton instance
     * @return True if registration was successful
     */
    template<typename T>
    bool registerQmlSingleton(const char *uri, int major, int minor, const char *qmlName,
                             QObject *(*callback)(QQmlEngine *, QJSEngine *)) {
        qmlRegisterSingletonType<T>(uri, major, minor, qmlName, callback);
        return true;
    }

    /**
     * @brief Register a C++ instance as a QML context property
     * @param name The name to use in QML
     * @param object The C++ object instance
     * @return True if registration was successful
     */
    bool registerContextProperty(const QString &name, QObject *object);

    /**
     * @brief Load a QML file
     * @param url The URL of the QML file
     * @return The root QObject of the loaded QML, or nullptr if loading failed
     */
    QObject *loadQml(const QUrl &url);

    /**
     * @brief Create a QML component
     * @param qml The QML source code
     * @return The created QObject, or nullptr if creation failed
     */
    QObject *createQmlComponent(const QString &qml);
    
    /**
     * @brief Get the current theme name
     * @return The current theme name
     */
    QString currentTheme() const;
    
    /**
     * @brief Set the current theme
     * @param themeName The theme name
     */
    void setCurrentTheme(const QString &themeName);
    
    /**
     * @brief Get all available themes
     * @return List of theme names
     */
    QStringList availableThemes() const;
    
    /**
     * @brief Create a new theme
     * @param name Theme name
     * @param baseTheme Base theme name (optional)
     * @return True if creation was successful
     */
    bool createTheme(const QString &name, const QString &baseTheme = QString());
    
    /**
     * @brief Delete a theme
     * @param name Theme name
     * @return True if deletion was successful
     */
    bool deleteTheme(const QString &name);
    
    /**
     * @brief Check if dark mode is enabled
     * @return True if dark mode is enabled
     */
    bool isDarkMode() const;
    
    /**
     * @brief Enable or disable dark mode
     * @param enabled Whether dark mode should be enabled
     */
    void setDarkMode(bool enabled);
    
    /**
     * @brief Get the accent color
     * @return The accent color
     */
    QColor accentColor() const;
    
    /**
     * @brief Set the accent color
     * @param color The new accent color
     */
    void setAccentColor(const QColor &color);
    
    /**
     * @brief Get the system font
     * @return The system font
     */
    QFont systemFont() const;
    
    /**
     * @brief Set the system font
     * @param font The new system font
     */
    void setSystemFont(const QFont &font);
    
    /**
     * @brief Get the UI scale factor
     * @return The UI scale factor (100 = 100%)
     */
    int uiScale() const;
    
    /**
     * @brief Set the UI scale factor
     * @param scale The new UI scale factor (100 = 100%)
     */
    void setUiScale(int scale);
    
    /**
     * @brief Check if accessibility mode is enabled
     * @return True if accessibility mode is enabled
     */
    bool accessibilityMode() const;
    
    /**
     * @brief Enable or disable accessibility mode
     * @param enabled Whether accessibility mode should be enabled
     */
    void setAccessibilityMode(bool enabled);
    
    /**
     * @brief Get the current language
     * @return The current language code (e.g., "en", "de")
     */
    QString language() const;
    
    /**
     * @brief Set the current language
     * @param languageCode The language code (e.g., "en", "de")
     */
    void setLanguage(const QString &languageCode);
    
    /**
     * @brief Get all available languages
     * @return Map of language codes to language names
     */
    QMap<QString, QString> availableLanguages() const;
    
    /**
     * @brief Create a panel
     * @param id Panel ID
     * @param position Panel position ("top", "bottom", "left", "right")
     * @param size Panel size in pixels
     * @return The created panel, or nullptr if creation failed
     */
    Panel* createPanel(const QString &id, const QString &position, int size);
    
    /**
     * @brief Get a panel by ID
     * @param id Panel ID
     * @return The panel, or nullptr if not found
     */
    Panel* getPanel(const QString &id) const;
    
    /**
     * @brief Get all panels
     * @return List of all panels
     */
    QList<Panel*> getAllPanels() const;
    
    /**
     * @brief Remove a panel
     * @param id Panel ID
     * @return True if removal was successful
     */
    bool removePanel(const QString &id);
    
    /**
     * @brief Create a widget
     * @param type Widget type
     * @param id Widget ID
     * @param properties Widget properties
     * @return The created widget, or nullptr if creation failed
     */
    Widget* createWidget(const QString &type, const QString &id, const QVariantMap &properties = QVariantMap());
    
    /**
     * @brief Get a widget by ID
     * @param id Widget ID
     * @return The widget, or nullptr if not found
     */
    Widget* getWidget(const QString &id) const;
    
    /**
     * @brief Get all widgets
     * @return List of all widgets
     */
    QList<Widget*> getAllWidgets() const;
    
    /**
     * @brief Remove a widget
     * @param id Widget ID
     * @return True if removal was successful
     */
    bool removeWidget(const QString &id);
    
    /**
     * @brief Add a widget to a panel
     * @param widgetId Widget ID
     * @param panelId Panel ID
     * @param position Position in the panel (index or "start", "center", "end")
     * @return True if addition was successful
     */
    bool addWidgetToPanel(const QString &widgetId, const QString &panelId, const QVariant &position);
    
    /**
     * @brief Remove a widget from a panel
     * @param widgetId Widget ID
     * @param panelId Panel ID
     * @return True if removal was successful
     */
    bool removeWidgetFromPanel(const QString &widgetId, const QString &panelId);
    
    /**
     * @brief Get all available widget types
     * @return List of widget type names
     */
    QStringList availableWidgetTypes() const;
    
    /**
     * @brief Register a widget factory
     * @param type Widget type
     * @param factory Factory function
     * @return True if registration was successful
     */
    bool registerWidgetFactory(const QString &type, std::function<Widget*(const QString&, const QVariantMap&)> factory);
    
    /**
     * @brief Save the current UI configuration
     * @param name Configuration name
     * @return True if saving was successful
     */
    bool saveConfiguration(const QString &name);
    
    /**
     * @brief Load a UI configuration
     * @param name Configuration name
     * @return True if loading was successful
     */
    bool loadConfiguration(const QString &name);
    
    /**
     * @brief Get all saved UI configurations
     * @return List of configuration names
     */
    QStringList savedConfigurations() const;
    
    /**
     * @brief Delete a saved UI configuration
     * @param name Configuration name
     * @return True if deletion was successful
     */
    bool deleteConfiguration(const QString &name);
    
    /**
     * @brief Show a notification
     * @param title Notification title
     * @param message Notification message
     * @param icon Notification icon (optional)
     * @param timeout Timeout in milliseconds (0 for no timeout)
     * @return Notification ID
     */
    int showNotification(const QString &title, const QString &message, const QUrl &icon = QUrl(), int timeout = 5000);
    
    /**
     * @brief Close a notification
     * @param id Notification ID
     * @return True if closing was successful
     */
    bool closeNotification(int id);

signals:
    /**
     * @brief Signal emitted when the UI is ready
     */
    void uiReady();
    
    /**
     * @brief Signal emitted when the current theme changes
     * @param themeName The new theme name
     */
    void currentThemeChanged(const QString &themeName);
    
    /**
     * @brief Signal emitted when dark mode changes
     * @param enabled Whether dark mode is enabled
     */
    void darkModeChanged(bool enabled);
    
    /**
     * @brief Signal emitted when the accent color changes
     * @param color The new accent color
     */
    void accentColorChanged(const QColor &color);
    
    /**
     * @brief Signal emitted when the system font changes
     * @param font The new system font
     */
    void systemFontChanged(const QFont &font);
    
    /**
     * @brief Signal emitted when the UI scale changes
     * @param scale The new UI scale
     */
    void uiScaleChanged(int scale);
    
    /**
     * @brief Signal emitted when accessibility mode changes
     * @param enabled Whether accessibility mode is enabled
     */
    void accessibilityModeChanged(bool enabled);
    
    /**
     * @brief Signal emitted when the language changes
     * @param languageCode The new language code
     */
    void languageChanged(const QString &languageCode);
    
    /**
     * @brief Signal emitted when a panel is created
     * @param panel The created panel
     */
    void panelCreated(Panel *panel);
    
    /**
     * @brief Signal emitted when a panel is removed
     * @param id The panel ID
     */
    void panelRemoved(const QString &id);
    
    /**
     * @brief Signal emitted when a widget is created
     * @param widget The created widget
     */
    void widgetCreated(Widget *widget);
    
    /**
     * @brief Signal emitted when a widget is removed
     * @param id The widget ID
     */
    void widgetRemoved(const QString &id);
    
    /**
     * @brief Signal emitted when a notification is shown
     * @param id The notification ID
     * @param title The notification title
     * @param message The notification message
     */
    void notificationShown(int id, const QString &title, const QString &message);
    
    /**
     * @brief Signal emitted when a notification is closed
     * @param id The notification ID
     */
    void notificationClosed(int id);

private:
    // The QML engine
    QQmlEngine *m_qmlEngine;
    
    // Current theme
    std::shared_ptr<Theme> m_currentTheme;
    QString m_currentThemeName;
    
    // UI settings
    bool m_darkMode;
    QColor m_accentColor;
    QFont m_systemFont;
    int m_uiScale;
    bool m_accessibilityMode;
    QString m_language;
    
    // Panels
    QMap<QString, Panel*> m_panels;
    
    // Widgets
    QMap<QString, Widget*> m_widgets;
    
    // Widget factories
    QMap<QString, std::function<Widget*(const QString&, const QVariantMap&)>> m_widgetFactories;
    
    // Notification counter
    int m_nextNotificationId;
    
    // Register all QML types
    void registerQmlTypes();
    
    // Register all context properties
    void registerContextProperties();
    
    // Load default theme
    void loadDefaultTheme();
    
    // Load available themes
    void loadAvailableThemes();
    
    // Load available languages
    void loadAvailableLanguages();
    
    // Register built-in widget factories
    void registerBuiltInWidgetFactories();
};

} // namespace VivoX::UI
