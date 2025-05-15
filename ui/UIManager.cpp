#include "UIManager.h"
#include "panels/Panel.h"
#include "widgets/Widget.h"
#include "config/Theme.h"

#include <QDebug>
#include <QQmlComponent>
#include <QQmlContext>
#include <QQuickItem>
#include <QDir>
#include <QFontDatabase>
#include <QTranslator>
#include <QCoreApplication>
#include <QSettings>

namespace VivoX::UI {

UIManager::UIManager(QObject *parent)
    : QObject(parent)
    , m_qmlEngine(nullptr)
    , m_currentThemeName("default")
    , m_darkMode(false)
    , m_accentColor(0, 120, 215) // Default blue accent color
    , m_systemFont("Noto Sans", 10)
    , m_uiScale(100)
    , m_accessibilityMode(false)
    , m_language("en")
    , m_nextNotificationId(1)
{
    qDebug() << "UIManager created";
}

UIManager::~UIManager()
{
    shutdown();
    qDebug() << "UIManager destroyed";
}

bool UIManager::initialize()
{
    // Create the QML engine
    m_qmlEngine = new QQmlEngine(this);
    
    if (!m_qmlEngine) {
        qWarning() << "Failed to create QML engine";
        return false;
    }
    
    // Register QML types
    registerQmlTypes();
    
    // Register context properties
    registerContextProperties();
    
    // Load default theme
    loadDefaultTheme();
    
    // Load available themes
    loadAvailableThemes();
    
    // Load available languages
    loadAvailableLanguages();
    
    // Register built-in widget factories
    registerBuiltInWidgetFactories();
    
    // Load settings
    QSettings settings("VivoX", "Desktop");
    
    // Load theme
    QString themeName = settings.value("ui/theme", "default").toString();
    if (themeName != "default") {
        setCurrentTheme(themeName);
    }
    
    // Load dark mode
    setDarkMode(settings.value("ui/darkMode", false).toBool());
    
    // Load accent color
    QColor color = settings.value("ui/accentColor", QColor(0, 120, 215)).value<QColor>();
    setAccentColor(color);
    
    // Load system font
    QFont font = settings.value("ui/systemFont", QFont("Noto Sans", 10)).value<QFont>();
    setSystemFont(font);
    
    // Load UI scale
    setUiScale(settings.value("ui/scale", 100).toInt());
    
    // Load accessibility mode
    setAccessibilityMode(settings.value("ui/accessibilityMode", false).toBool());
    
    // Load language
    QString lang = settings.value("ui/language", "en").toString();
    setLanguage(lang);
    
    qDebug() << "UIManager initialized";
    
    emit uiReady();
    
    return true;
}

void UIManager::shutdown()
{
    // Save settings
    QSettings settings("VivoX", "Desktop");
    
    settings.setValue("ui/theme", m_currentThemeName);
    settings.setValue("ui/darkMode", m_darkMode);
    settings.setValue("ui/accentColor", m_accentColor);
    settings.setValue("ui/systemFont", m_systemFont);
    settings.setValue("ui/scale", m_uiScale);
    settings.setValue("ui/accessibilityMode", m_accessibilityMode);
    settings.setValue("ui/language", m_language);
    
    // Clear panels
    qDeleteAll(m_panels);
    m_panels.clear();
    
    // Clear widgets
    qDeleteAll(m_widgets);
    m_widgets.clear();
    
    // Clear widget factories
    m_widgetFactories.clear();
    
    // Delete QML engine
    if (m_qmlEngine) {
        delete m_qmlEngine;
        m_qmlEngine = nullptr;
    }
}

QQmlEngine *UIManager::qmlEngine() const
{
    return m_qmlEngine;
}

bool UIManager::registerContextProperty(const QString &name, QObject *object)
{
    if (!m_qmlEngine) {
        qWarning() << "QML engine not initialized";
        return false;
    }
    
    if (!object) {
        qWarning() << "Cannot register null object as context property:" << name;
        return false;
    }
    
    m_qmlEngine->rootContext()->setContextProperty(name, object);
    
    qDebug() << "Registered context property:" << name;
    
    return true;
}

QObject *UIManager::loadQml(const QUrl &url)
{
    if (!m_qmlEngine) {
        qWarning() << "QML engine not initialized";
        return nullptr;
    }
    
    QQmlComponent component(m_qmlEngine, url);
    
    if (component.isError()) {
        qWarning() << "Failed to load QML file:" << url;
        for (const QQmlError &error : component.errors()) {
            qWarning() << error.toString();
        }
        return nullptr;
    }
    
    QObject *object = component.create(m_qmlEngine->rootContext());
    
    if (!object) {
        qWarning() << "Failed to create QML object from file:" << url;
        return nullptr;
    }
    
    qDebug() << "Loaded QML file:" << url;
    
    return object;
}

QObject *UIManager::createQmlComponent(const QString &qml)
{
    if (!m_qmlEngine) {
        qWarning() << "QML engine not initialized";
        return nullptr;
    }
    
    QQmlComponent component(m_qmlEngine);
    component.setData(qml.toUtf8(), QUrl());
    
    if (component.isError()) {
        qWarning() << "Failed to create QML component";
        for (const QQmlError &error : component.errors()) {
            qWarning() << error.toString();
        }
        return nullptr;
    }
    
    QObject *object = component.create(m_qmlEngine->rootContext());
    
    if (!object) {
        qWarning() << "Failed to create QML object from component";
        return nullptr;
    }
    
    qDebug() << "Created QML component";
    
    return object;
}

QString UIManager::currentTheme() const
{
    return m_currentThemeName;
}

void UIManager::setCurrentTheme(const QString &themeName)
{
    if (m_currentThemeName == themeName) {
        return;
    }
    
    // Check if theme exists
    if (!availableThemes().contains(themeName)) {
        qWarning() << "Theme not found:" << themeName;
        return;
    }
    
    // Load theme
    std::shared_ptr<Theme> theme = std::make_shared<Theme>(themeName);
    if (!theme->load()) {
        qWarning() << "Failed to load theme:" << themeName;
        return;
    }
    
    // Set theme
    m_currentTheme = theme;
    m_currentThemeName = themeName;
    
    // Update QML context
    if (m_qmlEngine) {
        m_qmlEngine->rootContext()->setContextProperty("theme", QVariant::fromValue(m_currentTheme.get()));
    }
    
    // Update dark mode based on theme
    setDarkMode(m_currentTheme->isDarkTheme());
    
    emit currentThemeChanged(themeName);
    
    qDebug() << "Set current theme:" << themeName;
}

QStringList UIManager::availableThemes() const
{
    QStringList themes;
    
    // Add default theme
    themes << "default";
    
    // Add custom themes
    QDir themesDir(QDir::homePath() + "/.config/vivox/themes");
    if (themesDir.exists()) {
        QStringList entries = themesDir.entryList(QDir::Dirs | QDir::NoDotAndDotDot);
        themes.append(entries);
    }
    
    return themes;
}

bool UIManager::createTheme(const QString &name, const QString &baseTheme)
{
    if (name.isEmpty()) {
        qWarning() << "Theme name cannot be empty";
        return false;
    }
    
    // Check if theme already exists
    if (availableThemes().contains(name)) {
        qWarning() << "Theme already exists:" << name;
        return false;
    }
    
    // Create theme directory
    QDir themesDir(QDir::homePath() + "/.config/vivox/themes");
    if (!themesDir.exists()) {
        themesDir.mkpath(".");
    }
    
    QDir themeDir(themesDir.filePath(name));
    if (!themeDir.exists()) {
        if (!themeDir.mkpath(".")) {
            qWarning() << "Failed to create theme directory:" << themeDir.path();
            return false;
        }
    }
    
    // Create theme file
    QFile themeFile(themeDir.filePath("theme.json"));
    if (!themeFile.open(QIODevice::WriteOnly | QIODevice::Text)) {
        qWarning() << "Failed to create theme file:" << themeFile.fileName();
        return false;
    }
    
    // Write theme data
    QJsonObject themeData;
    themeData["name"] = name;
    themeData["baseTheme"] = baseTheme.isEmpty() ? "default" : baseTheme;
    themeData["darkTheme"] = false;
    themeData["accentColor"] = "#0078d7";
    
    QJsonDocument doc(themeData);
    themeFile.write(doc.toJson());
    themeFile.close();
    
    qDebug() << "Created theme:" << name;
    
    return true;
}

bool UIManager::deleteTheme(const QString &name)
{
    if (name.isEmpty() || name == "default") {
        qWarning() << "Cannot delete default theme";
        return false;
    }
    
    // Check if theme exists
    if (!availableThemes().contains(name)) {
        qWarning() << "Theme not found:" << name;
        return false;
    }
    
    // Check if theme is current
    if (m_currentThemeName == name) {
        qWarning() << "Cannot delete current theme";
        return false;
    }
    
    // Delete theme directory
    QDir themesDir(QDir::homePath() + "/.config/vivox/themes");
    QDir themeDir(themesDir.filePath(name));
    
    if (!themeDir.removeRecursively()) {
        qWarning() << "Failed to delete theme directory:" << themeDir.path();
        return false;
    }
    
    qDebug() << "Deleted theme:" << name;
    
    return true;
}

bool UIManager::isDarkMode() const
{
    return m_darkMode;
}

void UIManager::setDarkMode(bool enabled)
{
    if (m_darkMode == enabled) {
        return;
    }
    
    m_darkMode = enabled;
    
    // Update QML context
    if (m_qmlEngine) {
        m_qmlEngine->rootContext()->setContextProperty("darkMode", m_darkMode);
    }
    
    emit darkModeChanged(enabled);
    
    qDebug() << "Set dark mode:" << (enabled ? "enabled" : "disabled");
}

QColor UIManager::accentColor() const
{
    return m_accentColor;
}

void UIManager::setAccentColor(const QColor &color)
{
    if (m_accentColor == color) {
        return;
    }
    
    m_accentColor = color;
    
    // Update QML context
    if (m_qmlEngine) {
        m_qmlEngine->rootContext()->setContextProperty("accentColor", m_accentColor);
    }
    
    emit accentColorChanged(color);
    
    qDebug() << "Set accent color:" << color;
}

QFont UIManager::systemFont() const
{
    return m_systemFont;
}

void UIManager::setSystemFont(const QFont &font)
{
    if (m_systemFont == font) {
        return;
    }
    
    m_systemFont = font;
    
    // Update QML context
    if (m_qmlEngine) {
        m_qmlEngine->rootContext()->setContextProperty("systemFont", m_systemFont);
    }
    
    emit systemFontChanged(font);
    
    qDebug() << "Set system font:" << font.family() << font.pointSize();
}

int UIManager::uiScale() const
{
    return m_uiScale;
}

void UIManager::setUiScale(int scale)
{
    if (m_uiScale == scale) {
        return;
    }
    
    // Validate scale
    if (scale < 50 || scale > 200) {
        qWarning() << "Invalid UI scale:" << scale;
        return;
    }
    
    m_uiScale = scale;
    
    // Update QML context
    if (m_qmlEngine) {
        m_qmlEngine->rootContext()->setContextProperty("uiScale", m_uiScale);
    }
    
    emit uiScaleChanged(scale);
    
    qDebug() << "Set UI scale:" << scale;
}

bool UIManager::accessibilityMode() const
{
    return m_accessibilityMode;
}

void UIManager::setAccessibilityMode(bool enabled)
{
    if (m_accessibilityMode == enabled) {
        return;
    }
    
    m_accessibilityMode = enabled;
    
    // Update QML context
    if (m_qmlEngine) {
        m_qmlEngine->rootContext()->setContextProperty("accessibilityMode", m_accessibilityMode);
    }
    
    emit accessibilityModeChanged(enabled);
    
    qDebug() << "Set accessibility mode:" << (enabled ? "enabled" : "disabled");
}

QString UIManager::language() const
{
    return m_language;
}

void UIManager::setLanguage(const QString &languageCode)
{
    if (m_language == languageCode) {
        return;
    }
    
    // Check if language exists
    if (!availableLanguages().contains(languageCode)) {
        qWarning() << "Language not found:" << languageCode;
        return;
    }
    
    m_language = languageCode;
    
    // Load translation
    static QTranslator translator;
    QCoreApplication::removeTranslator(&translator);
    
    if (languageCode != "en") {
        QString translationFile = QString(":/translations/vivox_%1.qm").arg(languageCode);
        if (translator.load(translationFile)) {
            QCoreApplication::installTranslator(&translator);
        } else {
            qWarning() << "Failed to load translation:" << translationFile;
        }
    }
    
    // Update QML context
    if (m_qmlEngine) {
        m_qmlEngine->rootContext()->setContextProperty("language", m_language);
    }
    
    emit languageChanged(languageCode);
    
    qDebug() << "Set language:" << languageCode;
}

QMap<QString, QString> UIManager::availableLanguages() const
{
    QMap<QString, QString> languages;
    
    // Add English (default)
    languages["en"] = "English";
    
    // Add other languages
    languages["de"] = "Deutsch";
    languages["fr"] = "Français";
    languages["es"] = "Español";
    languages["it"] = "Italiano";
    languages["ja"] = "日本語";
    languages["ko"] = "한국어";
    languages["zh"] = "中文";
    languages["ru"] = "Русский";
    
    return languages;
}

Panel* UIManager::createPanel(const QString &id, const QString &position, int size)
{
    if (id.isEmpty()) {
        qWarning() << "Panel ID cannot be empty";
        return nullptr;
    }
    
    // Check if panel already exists
    if (m_panels.contains(id)) {
        qWarning() << "Panel already exists:" << id;
        return nullptr;
    }
    
    // Validate position
    if (position != "top" && position != "bottom" && position != "left" && position != "right") {
        qWarning() << "Invalid panel position:" << position;
        return nullptr;
    }
    
    // Validate size
    if (size <= 0) {
        qWarning() << "Invalid panel size:" << size;
        return nullptr;
    }
    
    // Create panel
    Panel *panel = new Panel(id, position, size, this);
    
    // Add to list
    m_panels[id] = panel;
    
    emit panelCreated(panel);
    
    qDebug() << "Created panel:" << id << position << size;
    
    return panel;
}

Panel* UIManager::getPanel(const QString &id) const
{
    return m_panels.value(id, nullptr);
}

QList<Panel*> UIManager::getAllPanels() const
{
    return m_panels.values();
}

bool UIManager::removePanel(const QString &id)
{
    if (!m_panels.contains(id)) {
        qWarning() << "Panel not found:" << id;
        return false;
    }
    
    // Get panel
    Panel *panel = m_panels[id];
    
    // Remove from list
    m_panels.remove(id);
    
    // Delete panel
    delete panel;
    
    emit panelRemoved(id);
    
    qDebug() << "Removed panel:" << id;
    
    return true;
}

Widget* UIManager::createWidget(const QString &type, const QString &id, const QVariantMap &properties)
{
    if (id.isEmpty()) {
        qWarning() << "Widget ID cannot be empty";
        return nullptr;
    }
    
    // Check if widget already exists
    if (m_widgets.contains(id)) {
        qWarning() << "Widget already exists:" << id;
        return nullptr;
    }
    
    // Check if widget type exists
    if (!m_widgetFactories.contains(type)) {
        qWarning() << "Widget type not found:" << type;
        return nullptr;
    }
    
    // Create widget
    Widget *widget = m_widgetFactories[type](id, properties);
    
    if (!widget) {
        qWarning() << "Failed to create widget:" << id << type;
        return nullptr;
    }
    
    // Add to list
    m_widgets[id] = widget;
    
    emit widgetCreated(widget);
    
    qDebug() << "Created widget:" << id << type;
    
    return widget;
}

Widget* UIManager::getWidget(const QString &id) const
{
    return m_widgets.value(id, nullptr);
}

QList<Widget*> UIManager::getAllWidgets() const
{
    return m_widgets.values();
}

bool UIManager::removeWidget(const QString &id)
{
    if (!m_widgets.contains(id)) {
        qWarning() << "Widget not found:" << id;
        return false;
    }
    
    // Get widget
    Widget *widget = m_widgets[id];
    
    // Remove from panels
    for (Panel *panel : m_panels.values()) {
        panel->removeWidget(widget);
  
(Content truncated due to size limit. Use line ranges to read in chunks)