// src/ui/animations/AnimationManager.qml
pragma Singleton
import QtQuick 2.15

QtObject {
    id: animationManager

    // Animation curves
    readonly property var curves: {
        "standard": Easing.InOutQuad,
        "decelerate": Easing.OutCubic,
        "accelerate": Easing.InCubic,
        "sharp": Easing.OutQuint,
        "elastic": Easing.OutElastic,
        "bounce": Easing.OutBounce,
        "overshoot": Easing.OutBack
    }

    // Duration presets
    readonly property int instant: 0
    readonly property int extraFast: 100
    readonly property int fast: 150
    readonly property int normal: 250
    readonly property int slow: 350
    readonly property int extraSlow: 500

    // System animation settings
    property bool animationsEnabled: true
    property real animationScale: 1.0

    // Animation types
    readonly property string fadeIn: "fadeIn"
    readonly property string fadeOut: "fadeOut"
    readonly property string slideIn: "slideIn"
    readonly property string slideOut: "slideOut"
    readonly property string scaleIn: "scaleIn"
    readonly property string scaleOut: "scaleOut"

    // Get scaled duration
    function scaledDuration(baseDuration) {
        return animationsEnabled ? baseDuration * animationScale : 0
    }

    // Apply animation
    function applyAnimation(item, type, direction, options) {
        if (!item) return null

            options = options || {}
            var duration = options.duration !== undefined ? options.duration : normal
            var easing = options.easing !== undefined ? options.easing : curves.standard
            var delay = options.delay !== undefined ? options.delay : 0

            var animation

            switch (type) {
                case fadeIn:
                    animation = createFadeAnimation(item, 0, 1, duration, easing, delay)
                    break

                case fadeOut:
                    animation = createFadeAnimation(item, 1, 0, duration, easing, delay)
                    break

                case slideIn:
                    animation = createSlideAnimation(item, direction, true, duration, easing, delay)
                    break

                case slideOut:
                    animation = createSlideAnimation(item, direction, false, duration, easing, delay)
                    break

                case scaleIn:
                    animation = createScaleAnimation(item, 0, 1, duration, easing, delay)
                    break

                case scaleOut:
                    animation = createScaleAnimation(item, 1, 0, duration, easing, delay)
                    break

                default:
                    console.error("Unknown animation type:", type)
                    return null
            }

            if (options.onCompleted) {
                animation.onStopped.connect(options.onCompleted)
            }

            animation.start()
            return animation
    }

    // Helper functions
    function createFadeAnimation(item, from, to, duration, easing, delay) {
        var animation = Qt.createQmlObject(`
        import QtQuick 2.15

        NumberAnimation {
            target: ${item}
            property: "opacity"
            from: ${from}
            to: ${to}
            duration: ${scaledDuration(duration)}
            easing.type: ${easing}
            running: false
        }
        `, item)

        return animation
    }

    function createSlideAnimation(item, direction, isIn, duration, easing, delay) {
        var prop, from, to

        if (direction === "left" || direction === "right") {
            prop = "x"
            if (direction === "left") {
                from = isIn ? -item.width : 0
                to = isIn ? 0 : -item.width
            } else {
                from = isIn ? item.parent.width : 0
                to = isIn ? 0 : item.parent.width
            }
        } else {
            prop = "y"
            if (direction === "up") {
                from = isIn ? -item.height : 0
                to = isIn ? 0 : -item.height
            } else {
                from = isIn ? item.parent.height : 0
                to = isIn ? 0 : item.parent.height
            }
        }

        var animation = Qt.createQmlObject(`
        import QtQuick 2.15

        NumberAnimation {
            target: ${item}
            property: "${prop}"
            from: ${from}
            to: ${to}
            duration: ${scaledDuration(duration)}
            easing.type: ${easing}
            running: false
        }
        `, item)

        return animation
    }

    function createScaleAnimation(item, from, to, duration, easing, delay) {
        var animation = Qt.createQmlObject(`
        import QtQuick 2.15

        NumberAnimation {
            target: ${item}
            property: "scale"
            from: ${from}
            to: ${to}
            duration: ${scaledDuration(duration)}
            easing.type: ${easing}
            running: false
        }
        `, item)

        return animation
    }
}
