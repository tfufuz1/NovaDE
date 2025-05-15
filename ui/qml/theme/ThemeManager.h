#pragma once

#include <QObject>
#include <QString>
#include <QQmlEngine>
#include <QQuickItem>
#include <QColor>

namespace VivoX::UI {

/**
 * @brief The ThemeManager class manages the UI theme and styling.
 * 
 * It provides a centralized way to manage colors, fonts, and other styling
 * properties, and allows for theme switching and customization.
 */
class ThemeManager : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString themeName READ themeName WRITE setThemeName NOTIFY themeNameChanged)
    Q_PROPERTY(bool darkMode READ darkMode WRITE setDarkMode NOTIFY darkModeChanged)
    Q_PROPERTY(QColor accentColor READ accentColor WRITE setAccentColor NOTIFY accentColorChanged)
    Q_PROPERTY(QColor backgroundColor READ backgroundColor NOTIFY backgroundColorChanged)
    Q_PROPERTY(QColor foregroundColor READ foregroundColor NOTIFY foregroundColorChanged)
    Q_PROPERTY(QColor textColor READ textColor NOTIFY textColorChanged)
    Q_PROPERTY(QColor primaryColor READ primaryColor NOTIFY primaryColorChanged)
    Q_PROPERTY(QColor secondaryColor READ secondaryColor NOTIFY secondaryColorChanged)
    Q_PROPERTY(QColor highlightColor READ highlightColor NOTIFY highlightColorChanged)
    Q_PROPERTY(QColor shadowColor READ shadowColor NOTIFY shadowColorChanged)
    Q_PROPERTY(int fontSize READ fontSize WRITE setFontSize NOTIFY fontSizeChanged)
    Q_PROPERTY(QString fontFamily READ fontFamily WRITE setFontFamily NOTIFY fontFamilyChanged)
    Q_PROPERTY(int cornerRadius READ cornerRadius WRITE setCornerRadius NOTIFY cornerRadiusChanged)
    Q_PROPERTY(int shadowRadius READ shadowRadius WRITE setShadowRadius NOTIFY shadowRadiusChanged)
    Q_PROPERTY(int spacing READ spacing WRITE setSpacing NOTIFY spacingChanged)
    Q_PROPERTY(int padding READ padding WRITE setPadding NOTIFY paddingChanged)

public:
    explicit ThemeManager(QObject *parent = nullptr);
    ~ThemeManager();

    /**
     * @brief Initialize the theme manager
     * @return True if initialization was successful
     */
    bool initialize();

    /**
     * @brief Get the current theme name
     * @return The theme name
     */
    QString themeName() const;

    /**
     * @brief Set the current theme name
     * @param themeName The theme name
     */
    void setThemeName(const QString &themeName);

    /**
     * @brief Check if dark mode is enabled
     * @return True if dark mode is enabled
     */
    bool darkMode() const;

    /**
     * @brief Set dark mode
     * @param darkMode Whether to enable dark mode
     */
    void setDarkMode(bool darkMode);

    /**
     * @brief Get the accent color
     * @return The accent color
     */
    QColor accentColor() const;

    /**
     * @brief Set the accent color
     * @param color The accent color
     */
    void setAccentColor(const QColor &color);

    /**
     * @brief Get the background color
     * @return The background color
     */
    QColor backgroundColor() const;

    /**
     * @brief Get the foreground color
     * @return The foreground color
     */
    QColor foregroundColor() const;

    /**
     * @brief Get the text color
     * @return The text color
     */
    QColor textColor() const;

    /**
     * @brief Get the primary color
     * @return The primary color
     */
    QColor primaryColor() const;

    /**
     * @brief Get the secondary color
     * @return The secondary color
     */
    QColor secondaryColor() const;

    /**
     * @brief Get the highlight color
     * @return The highlight color
     */
    QColor highlightColor() const;

    /**
     * @brief Get the shadow color
     * @return The shadow color
     */
    QColor shadowColor() const;

    /**
     * @brief Get the font size
     * @return The font size
     */
    int fontSize() const;

    /**
     * @brief Set the font size
     * @param size The font size
     */
    void setFontSize(int size);

    /**
     * @brief Get the font family
     * @return The font family
     */
    QString fontFamily() const;

    /**
     * @brief Set the font family
     * @param family The font family
     */
    void setFontFamily(const QString &family);

    /**
     * @brief Get the corner radius
     * @return The corner radius
     */
    int cornerRadius() const;

    /**
     * @brief Set the corner radius
     * @param radius The corner radius
     */
    void setCornerRadius(int radius);

    /**
     * @brief Get the shadow radius
     * @return The shadow radius
     */
    int shadowRadius() const;

    /**
     * @brief Set the shadow radius
     * @param radius The shadow radius
     */
    void setShadowRadius(int radius);

    /**
     * @brief Get the spacing
     * @return The spacing
     */
    int spacing() const;

    /**
     * @brief Set the spacing
     * @param spacing The spacing
     */
    void setSpacing(int spacing);

    /**
     * @brief Get the padding
     * @return The padding
     */
    int padding() const;

    /**
     * @brief Set the padding
     * @param padding The padding
     */
    void setPadding(int padding);

    /**
     * @brief Load a theme from a file
     * @param filePath The path to the theme file
     * @return True if loading was successful
     */
    Q_INVOKABLE bool loadTheme(const QString &filePath);

    /**
     * @brief Save the current theme to a file
     * @param filePath The path to save the theme file
     * @return True if saving was successful
     */
    Q_INVOKABLE bool saveTheme(const QString &filePath);

    /**
     * @brief Get a color by name
     * @param name The color name
     * @return The color
     */
    Q_INVOKABLE QColor getColor(const QString &name) const;

    /**
     * @brief Set a color by name
     * @param name The color name
     * @param color The color
     * @return True if setting was successful
     */
    Q_INVOKABLE bool setColor(const QString &name, const QColor &color);

signals:
    /**
     * @brief Signal emitted when the theme name changes
     * @param themeName The new theme name
     */
    void themeNameChanged(const QString &themeName);

    /**
     * @brief Signal emitted when the dark mode changes
     * @param darkMode The new dark mode state
     */
    void darkModeChanged(bool darkMode);

    /**
     * @brief Signal emitted when the accent color changes
     * @param color The new accent color
     */
    void accentColorChanged(const QColor &color);

    /**
     * @brief Signal emitted when the background color changes
     * @param color The new background color
     */
    void backgroundColorChanged(const QColor &color);

    /**
     * @brief Signal emitted when the foreground color changes
     * @param color The new foreground color
     */
    void foregroundColorChanged(const QColor &color);

    /**
     * @brief Signal emitted when the text color changes
     * @param color The new text color
     */
    void textColorChanged(const QColor &color);

    /**
     * @brief Signal emitted when the primary color changes
     * @param color The new primary color
     */
    void primaryColorChanged(const QColor &color);

    /**
     * @brief Signal emitted when the secondary color changes
     * @param color The new secondary color
     */
    void secondaryColorChanged(const QColor &color);

    /**
     * @brief Signal emitted when the highlight color changes
     * @param color The new highlight color
     */
    void highlightColorChanged(const QColor &color);

    /**
     * @brief Signal emitted when the shadow color changes
     * @param color The new shadow color
     */
    void shadowColorChanged(const QColor &color);

    /**
     * @brief Signal emitted when the font size changes
     * @param size The new font size
     */
    void fontSizeChanged(int size);

    /**
     * @brief Signal emitted when the font family changes
     * @param family The new font family
     */
    void fontFamilyChanged(const QString &family);

    /**
     * @brief Signal emitted when the corner radius changes
     * @param radius The new corner radius
     */
    void cornerRadiusChanged(int radius);

    /**
     * @brief Signal emitted when the shadow radius changes
     * @param radius The new shadow radius
     */
    void shadowRadiusChanged(int radius);

    /**
     * @brief Signal emitted when the spacing changes
     * @param spacing The new spacing
     */
    void spacingChanged(int spacing);

    /**
     * @brief Signal emitted when the padding changes
     * @param padding The new padding
     */
    void paddingChanged(int padding);

    /**
     * @brief Signal emitted when the theme changes
     */
    void themeChanged();

private:
    // Theme name
    QString m_themeName;
    
    // Dark mode
    bool m_darkMode;
    
    // Colors
    QColor m_accentColor;
    QColor m_backgroundColor;
    QColor m_foregroundColor;
    QColor m_textColor;
    QColor m_primaryColor;
    QColor m_secondaryColor;
    QColor m_highlightColor;
    QColor m_shadowColor;
    
    // Font properties
    int m_fontSize;
    QString m_fontFamily;
    
    // Styling properties
    int m_cornerRadius;
    int m_shadowRadius;
    int m_spacing;
    int m_padding;
    
    // Map of color names to colors
    QHash<QString, QColor> m_colors;
    
    // Apply the current theme
    void applyTheme();
    
    // Update derived colors
    void updateDerivedColors();
    
    // Load the default theme
    void loadDefaultTheme();
};

} // namespace VivoX::UI
