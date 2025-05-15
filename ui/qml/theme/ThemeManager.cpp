#include "ThemeManager.h"

#include <QDebug>
#include <QFile>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>

namespace VivoX::UI {

ThemeManager::ThemeManager(QObject *parent)
    : QObject(parent)
    , m_darkMode(false)
    , m_fontSize(12)
    , m_fontFamily("Noto Sans")
    , m_cornerRadius(4)
    , m_shadowRadius(8)
    , m_spacing(8)
    , m_padding(12)
{
    qDebug() << "ThemeManager created";
    
    // Load the default theme
    loadDefaultTheme();
}

ThemeManager::~ThemeManager()
{
    qDebug() << "ThemeManager destroyed";
}

bool ThemeManager::initialize()
{
    qDebug() << "ThemeManager initialized";
    return true;
}

QString ThemeManager::themeName() const
{
    return m_themeName;
}

void ThemeManager::setThemeName(const QString &themeName)
{
    if (m_themeName != themeName) {
        m_themeName = themeName;
        emit themeNameChanged(m_themeName);
        
        qDebug() << "Theme name changed:" << m_themeName;
    }
}

bool ThemeManager::darkMode() const
{
    return m_darkMode;
}

void ThemeManager::setDarkMode(bool darkMode)
{
    if (m_darkMode != darkMode) {
        m_darkMode = darkMode;
        
        // Update colors based on dark mode
        updateDerivedColors();
        
        emit darkModeChanged(m_darkMode);
        emit themeChanged();
        
        qDebug() << "Dark mode changed:" << m_darkMode;
    }
}

QColor ThemeManager::accentColor() const
{
    return m_accentColor;
}

void ThemeManager::setAccentColor(const QColor &color)
{
    if (m_accentColor != color) {
        m_accentColor = color;
        
        // Update derived colors
        updateDerivedColors();
        
        emit accentColorChanged(m_accentColor);
        emit themeChanged();
        
        qDebug() << "Accent color changed:" << m_accentColor.name();
    }
}

QColor ThemeManager::backgroundColor() const
{
    return m_backgroundColor;
}

QColor ThemeManager::foregroundColor() const
{
    return m_foregroundColor;
}

QColor ThemeManager::textColor() const
{
    return m_textColor;
}

QColor ThemeManager::primaryColor() const
{
    return m_primaryColor;
}

QColor ThemeManager::secondaryColor() const
{
    return m_secondaryColor;
}

QColor ThemeManager::highlightColor() const
{
    return m_highlightColor;
}

QColor ThemeManager::shadowColor() const
{
    return m_shadowColor;
}

int ThemeManager::fontSize() const
{
    return m_fontSize;
}

void ThemeManager::setFontSize(int size)
{
    if (m_fontSize != size) {
        m_fontSize = size;
        emit fontSizeChanged(m_fontSize);
        emit themeChanged();
        
        qDebug() << "Font size changed:" << m_fontSize;
    }
}

QString ThemeManager::fontFamily() const
{
    return m_fontFamily;
}

void ThemeManager::setFontFamily(const QString &family)
{
    if (m_fontFamily != family) {
        m_fontFamily = family;
        emit fontFamilyChanged(m_fontFamily);
        emit themeChanged();
        
        qDebug() << "Font family changed:" << m_fontFamily;
    }
}

int ThemeManager::cornerRadius() const
{
    return m_cornerRadius;
}

void ThemeManager::setCornerRadius(int radius)
{
    if (m_cornerRadius != radius) {
        m_cornerRadius = radius;
        emit cornerRadiusChanged(m_cornerRadius);
        emit themeChanged();
        
        qDebug() << "Corner radius changed:" << m_cornerRadius;
    }
}

int ThemeManager::shadowRadius() const
{
    return m_shadowRadius;
}

void ThemeManager::setShadowRadius(int radius)
{
    if (m_shadowRadius != radius) {
        m_shadowRadius = radius;
        emit shadowRadiusChanged(m_shadowRadius);
        emit themeChanged();
        
        qDebug() << "Shadow radius changed:" << m_shadowRadius;
    }
}

int ThemeManager::spacing() const
{
    return m_spacing;
}

void ThemeManager::setSpacing(int spacing)
{
    if (m_spacing != spacing) {
        m_spacing = spacing;
        emit spacingChanged(m_spacing);
        emit themeChanged();
        
        qDebug() << "Spacing changed:" << m_spacing;
    }
}

int ThemeManager::padding() const
{
    return m_padding;
}

void ThemeManager::setPadding(int padding)
{
    if (m_padding != padding) {
        m_padding = padding;
        emit paddingChanged(m_padding);
        emit themeChanged();
        
        qDebug() << "Padding changed:" << m_padding;
    }
}

bool ThemeManager::loadTheme(const QString &filePath)
{
    QFile file(filePath);
    
    if (!file.open(QIODevice::ReadOnly)) {
        qWarning() << "Failed to open theme file:" << filePath;
        return false;
    }
    
    QByteArray data = file.readAll();
    file.close();
    
    QJsonDocument doc = QJsonDocument::fromJson(data);
    
    if (doc.isNull() || !doc.isObject()) {
        qWarning() << "Invalid theme file format:" << filePath;
        return false;
    }
    
    QJsonObject obj = doc.object();
    
    // Load theme properties
    setThemeName(obj["name"].toString());
    setDarkMode(obj["darkMode"].toBool());
    setAccentColor(QColor(obj["accentColor"].toString()));
    setFontSize(obj["fontSize"].toInt());
    setFontFamily(obj["fontFamily"].toString());
    setCornerRadius(obj["cornerRadius"].toInt());
    setShadowRadius(obj["shadowRadius"].toInt());
    setSpacing(obj["spacing"].toInt());
    setPadding(obj["padding"].toInt());
    
    // Load colors
    QJsonObject colors = obj["colors"].toObject();
    for (auto it = colors.begin(); it != colors.end(); ++it) {
        setColor(it.key(), QColor(it.value().toString()));
    }
    
    qDebug() << "Loaded theme from file:" << filePath;
    
    return true;
}

bool ThemeManager::saveTheme(const QString &filePath)
{
    QJsonObject obj;
    
    // Save theme properties
    obj["name"] = m_themeName;
    obj["darkMode"] = m_darkMode;
    obj["accentColor"] = m_accentColor.name();
    obj["fontSize"] = m_fontSize;
    obj["fontFamily"] = m_fontFamily;
    obj["cornerRadius"] = m_cornerRadius;
    obj["shadowRadius"] = m_shadowRadius;
    obj["spacing"] = m_spacing;
    obj["padding"] = m_padding;
    
    // Save colors
    QJsonObject colors;
    for (auto it = m_colors.begin(); it != m_colors.end(); ++it) {
        colors[it.key()] = it.value().name();
    }
    obj["colors"] = colors;
    
    QJsonDocument doc(obj);
    
    QFile file(filePath);
    
    if (!file.open(QIODevice::WriteOnly)) {
        qWarning() << "Failed to open theme file for writing:" << filePath;
        return false;
    }
    
    file.write(doc.toJson());
    file.close();
    
    qDebug() << "Saved theme to file:" << filePath;
    
    return true;
}

QColor ThemeManager::getColor(const QString &name) const
{
    return m_colors.value(name, Qt::black);
}

bool ThemeManager::setColor(const QString &name, const QColor &color)
{
    if (name.isEmpty()) {
        qWarning() << "Cannot set color with empty name";
        return false;
    }
    
    m_colors[name] = color;
    
    // Update standard colors if this is one of them
    if (name == "accent") {
        setAccentColor(color);
    } else if (name == "background") {
        m_backgroundColor = color;
        emit backgroundColorChanged(m_backgroundColor);
    } else if (name == "foreground") {
        m_foregroundColor = color;
        emit foregroundColorChanged(m_foregroundColor);
    } else if (name == "text") {
        m_textColor = color;
        emit textColorChanged(m_textColor);
    } else if (name == "primary") {
        m_primaryColor = color;
        emit primaryColorChanged(m_primaryColor);
    } else if (name == "secondary") {
        m_secondaryColor = color;
        emit secondaryColorChanged(m_secondaryColor);
    } else if (name == "highlight") {
        m_highlightColor = color;
        emit highlightColorChanged(m_highlightColor);
    } else if (name == "shadow") {
        m_shadowColor = color;
        emit shadowColorChanged(m_shadowColor);
    } else {
        // For custom colors, just emit the theme changed signal
        emit themeChanged();
    }
    
    qDebug() << "Set color:" << name << color.name();
    
    return true;
}

void ThemeManager::applyTheme()
{
    // Apply the current theme to the UI
    // In a real implementation, this would update the QML style
    
    emit themeChanged();
    
    qDebug() << "Applied theme:" << m_themeName;
}

void ThemeManager::updateDerivedColors()
{
    // Update derived colors based on accent color and dark mode
    
    if (m_darkMode) {
        // Dark theme
        m_backgroundColor = QColor(30, 30, 30);
        m_foregroundColor = QColor(50, 50, 50);
        m_textColor = QColor(240, 240, 240);
        m_shadowColor = QColor(0, 0, 0, 100);
    } else {
        // Light theme
        m_backgroundColor = QColor(240, 240, 240);
        m_foregroundColor = QColor(255, 255, 255);
        m_textColor = QColor(30, 30, 30);
        m_shadowColor = QColor(0, 0, 0, 50);
    }
    
    // Derive colors from accent color
    m_primaryColor = m_accentColor;
    
    // Create a slightly darker version of the accent color for secondary
    QColor secondary = m_accentColor.darker(120);
    m_secondaryColor = secondary;
    
    // Create a slightly lighter version of the accent color for highlight
    QColor highlight = m_accentColor.lighter(120);
    m_highlightColor = highlight;
    
    // Update the color map
    m_colors["accent"] = m_accentColor;
    m_colors["background"] = m_backgroundColor;
    m_colors["foreground"] = m_foregroundColor;
    m_colors["text"] = m_textColor;
    m_colors["primary"] = m_primaryColor;
    m_colors["secondary"] = m_secondaryColor;
    m_colors["highlight"] = m_highlightColor;
    m_colors["shadow"] = m_shadowColor;
    
    // Emit signals for all changed colors
    emit backgroundColorChanged(m_backgroundColor);
    emit foregroundColorChanged(m_foregroundColor);
    emit textColorChanged(m_textColor);
    emit primaryColorChanged(m_primaryColor);
    emit secondaryColorChanged(m_secondaryColor);
    emit highlightColorChanged(m_highlightColor);
    emit shadowColorChanged(m_shadowColor);
    
    qDebug() << "Updated derived colors";
}

void ThemeManager::loadDefaultTheme()
{
    // Set default theme properties
    setThemeName("Default");
    setDarkMode(false);
    setAccentColor(QColor(0, 120, 215)); // Windows blue
    setFontSize(12);
    setFontFamily("Noto Sans");
    setCornerRadius(4);
    setShadowRadius(8);
    setSpacing(8);
    setPadding(12);
    
    // Derived colors will be updated by setAccentColor and setDarkMode
    
    qDebug() << "Loaded default theme";
}

} // namespace VivoX::UI
