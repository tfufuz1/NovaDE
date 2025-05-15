pragma Singleton
import QtQuick 2.15

QtObject {
    id: theme
    
    // Semantic color tokens
    readonly property color colorBackgroundPrimary: "#1A1A1A"
    readonly property color colorBackgroundSecondary: "#2C2C2C"
    readonly property color colorBackgroundTertiary: "#383838"
    readonly property color colorBackgroundHovered: "#404040"
    readonly property color colorBackgroundSelected: "#505050"
    
    readonly property color colorTextPrimary: "#E0E0E0"
    readonly property color colorTextSecondary: "#A0A0A0"
    readonly property color colorTextDisabled: "#707070"
    
    readonly property color colorAccentPrimary: "#3584e4"
    readonly property color colorAccentSecondary: "#62a0ea"
    
    readonly property color colorSemanticSuccess: "#2ec27e"
    readonly property color colorSemanticWarning: "#e5a50a"
    readonly property color colorSemanticError: "#e01b24"
    readonly property color colorSemanticInfo: "#3584e4"
    
    readonly property color colorBorderNormal: "#505050"
    readonly property color colorBorderLight: "#404040"
    readonly property color colorBorderFocus: "#3584e4"
    
    // Typography
    readonly property int fontSizeSmall: 12
    readonly property int fontSizeNormal: 14
    readonly property int fontSizeLarge: 16
    readonly property int fontSizeHeader: 20
    readonly property int fontSizeTitle: 24
    
    readonly property string fontFamily: "Noto Sans"
    
    // Metrics
    readonly property int spacingUnit: 8
    readonly property int cornerRadiusSmall: 4
    readonly property int cornerRadiusNormal: 8
    readonly property int cornerRadiusLarge: 12
    
    // Shadows and elevation
    readonly property var elevationLow: {
        "shadowColor": "#20000000",
        "shadowOffsetX": 0,
        "shadowOffsetY": 1,
        "shadowBlur": 2
    }
    
    readonly property var elevationMedium: {
        "shadowColor": "#30000000",
        "shadowOffsetX": 0,
        "shadowOffsetY": 2,
        "shadowBlur": 4
    }
    
    readonly property var elevationHigh: {
        "shadowColor": "#40000000",
        "shadowOffsetX": 0,
        "shadowOffsetY": 4,
        "shadowBlur": 8
    }
    
    // Animation durations
    readonly property int animationDurationFast: 150
    readonly property int animationDurationNormal: 250
    readonly property int animationDurationSlow: 350
    
    // File view modes
    readonly property int fileViewModeList: 0
    readonly property int fileViewModeGrid: 1
    readonly property int fileViewModeCompact: 2
    
    // Current theme
    property string activeTheme: "dark"
    
    function loadTheme(themeName) {
        // Load theme from configuration
        console.log("Loading theme:", themeName)
        activeTheme = themeName
    }
}
