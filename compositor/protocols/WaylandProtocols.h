/**
 * @file WaylandProtocols.h
 * @brief Definitionen für Wayland-Protokolle
 *
 * Diese Datei enthält die Definitionen für alle Wayland-Protokolle, die vom
 * VivoX Desktop Environment unterstützt werden.
 */

#pragma once

#include <QObject>
#include <QWaylandCompositor>
#include <QWaylandSurface>
#include <QWaylandSeat>
#include <QWaylandOutput>
#include <QWaylandXdgShell>
#include <QWaylandXdgDecorationManagerV1>
#include <QWaylandQtWindowManager>
#include <QWaylandIviApplication>
#include <QWaylandTextInputManager>
#include <QWaylandLinuxDmabufClientBufferIntegration>
#include <QMap>

namespace VivoX {
    namespace Wayland {

        // Vorwärtsdeklarationen
        class XWaylandManager;
        class LayerShellManager;
        class PresentationTimeManager;
        class ViewporterManager;

        /**
         * @class ProtocolManager
         * @brief Manager-Klasse für Wayland-Protokolle
         *
         * Diese Klasse ist für die Verwaltung und Registrierung aller Wayland-Protokolle
         * verantwortlich, die vom VivoX Desktop Environment unterstützt werden.
         */
        class ProtocolManager : public QObject {
            Q_OBJECT

        public:
            /**
             * @brief Gibt die Singleton-Instanz zurück
             * @return Singleton-Instanz
             */
            static ProtocolManager& instance();

            /**
             * @brief Destruktor
             */
            ~ProtocolManager();

            /**
             * @brief Initialisiert den Protocol-Manager
             * @param compositor Wayland-Compositor
             * @return true bei Erfolg, false bei Fehler
             */
            bool initialize(QWaylandCompositor* compositor);

            /**
             * @brief Registriert alle Standard-Protokolle
             * @return true bei Erfolg, false bei Fehler
             */
            bool registerStandardProtocols();

            /**
             * @brief Registriert alle Erweiterungs-Protokolle
             * @return true bei Erfolg, false bei Fehler
             */
            bool registerExtensionProtocols();

            /**
             * @brief Gibt den XDG-Shell zurück
             * @return XDG-Shell
             */
            QWaylandXdgShell* xdgShell() const;

            /**
             * @brief Gibt den XDG-Decoration-Manager zurück
             * @return XDG-Decoration-Manager
             */
            QWaylandXdgDecorationManagerV1* xdgDecorationManager() const;

            /**
             * @brief Gibt den Qt-Window-Manager zurück
             * @return Qt-Window-Manager
             */
            QWaylandQtWindowManager* qtWindowManager() const;

            /**
             * @brief Gibt den IVI-Application-Manager zurück
             * @return IVI-Application-Manager
             */
            QWaylandIviApplication* iviApplication() const;

            /**
             * @brief Gibt den Text-Input-Manager zurück
             * @return Text-Input-Manager
             */
            QWaylandTextInputManager* textInputManager() const;

            /**
             * @brief Gibt die Linux-DMABUF-Integration zurück
             * @return Linux-DMABUF-Integration
             */
            QWaylandLinuxDmabufClientBufferIntegration* linuxDmabuf() const;

            /**
             * @brief Gibt den Layer-Shell-Manager zurück
             * @return Layer-Shell-Manager
             */
            LayerShellManager* layerShell() const;

            /**
             * @brief Gibt den Presentation-Time-Manager zurück
             * @return Presentation-Time-Manager
             */
            PresentationTimeManager* presentationTime() const;

            /**
             * @brief Gibt den Viewporter-Manager zurück
             * @return Viewporter-Manager
             */
            ViewporterManager* viewporter() const;

            /**
             * @brief Gibt den XWayland-Manager zurück
             * @return XWayland-Manager
             */
            XWaylandManager* xwayland() const;

        signals:
            /**
             * @brief Signal, das ausgelöst wird, wenn ein Protokoll registriert wird
             * @param name Name des Protokolls
             */
            void protocolRegistered(const QString& name);

            /**
             * @brief Signal, das ausgelöst wird, wenn eine XDG-Surface erstellt wird
             * @param xdgSurface Die erstellte XDG-Surface
             */
            void xdgSurfaceCreated(QWaylandXdgSurface* xdgSurface);

            /**
             * @brief Signal, das ausgelöst wird, wenn eine XDG-Toplevel-Surface erstellt wird
             * @param toplevel Die erstellte XDG-Toplevel-Surface
             */
            void topLevelSurfaceCreated(QWaylandXdgToplevel* toplevel);

            /**
             * @brief Signal, das ausgelöst wird, wenn eine XDG-Popup-Surface erstellt wird
             * @param popup Die erstellte XDG-Popup-Surface
             */
            void popupSurfaceCreated(QWaylandXdgPopup* popup);

            /**
             * @brief Signal, das ausgelöst wird, wenn eine XDG-Decoration erstellt wird
             * @param decoration Die erstellte XDG-Decoration
             */
            void decorationCreated(QWaylandXdgToplevelDecorationV1* decoration);

            /**
             * @brief Signal, das ausgelöst wird, wenn eine IVI-Surface erstellt wird
             * @param iviSurface Die erstellte IVI-Surface
             */
            void iviSurfaceCreated(QWaylandIviSurface* iviSurface);

        private:
            /**
             * @brief Konstruktor
             * @param parent Elternobjekt
             */
            explicit ProtocolManager(QObject* parent = nullptr);

            /**
             * @brief Kopier-Konstruktor (nicht implementiert)
             */
            ProtocolManager(const ProtocolManager&) = delete;

            /**
             * @brief Zuweisungsoperator (nicht implementiert)
             */
            ProtocolManager& operator=(const ProtocolManager&) = delete;

            /**
             * @brief Registriert das XDG-Shell-Protokoll
             * @return true bei Erfolg, false bei Fehler
             */
            bool registerXdgShell();

            /**
             * @brief Registriert das XDG-Decoration-Protokoll
             * @return true bei Erfolg, false bei Fehler
             */
            bool registerXdgDecoration();

            /**
             * @brief Registriert das Qt-Window-Manager-Protokoll
             * @return true bei Erfolg, false bei Fehler
             */
            bool registerQtWindowManager();

            /**
             * @brief Registriert das IVI-Application-Protokoll
             * @return true bei Erfolg, false bei Fehler
             */
            bool registerIviApplication();

            /**
             * @brief Registriert das Text-Input-Protokoll
             * @return true bei Erfolg, false bei Fehler
             */
            bool registerTextInput();

            /**
             * @brief Registriert das Linux-DMABUF-Protokoll
             * @return true bei Erfolg, false bei Fehler
             */
            bool registerLinuxDmabuf();

            /**
             * @brief Registriert das Layer-Shell-Protokoll
             * @return true bei Erfolg, false bei Fehler
             */
            bool registerLayerShell();

            /**
             * @brief Registriert das Presentation-Time-Protokoll
             * @return true bei Erfolg, false bei Fehler
             */
            bool registerPresentationTime();

            /**
             * @brief Registriert das Viewporter-Protokoll
             * @return true bei Erfolg, false bei Fehler
             */
            bool registerViewporter();

            /**
             * @brief Registriert das XWayland-Protokoll
             * @return true bei Erfolg, false bei Fehler
             */
            bool registerXWayland();

            QWaylandCompositor* m_compositor; ///< Wayland-Compositor
            QWaylandXdgShell* m_xdgShell; ///< XDG-Shell
            QWaylandXdgDecorationManagerV1* m_xdgDecorationManager; ///< XDG-Decoration-Manager
            QWaylandQtWindowManager* m_qtWindowManager; ///< Qt-Window-Manager
            QWaylandIviApplication* m_iviApplication; ///< IVI-Application-Manager
            QWaylandTextInputManager* m_textInputManager; ///< Text-Input-Manager
            QWaylandLinuxDmabufClientBufferIntegration* m_linuxDmabuf; ///< Linux-DMABUF-Integration
            QObject* m_layerShell; ///< Layer-Shell-Manager
            QObject* m_presentationTime; ///< Presentation-Time-Manager
            QObject* m_viewporter; ///< Viewporter-Manager
            QObject* m_xwayland; ///< XWayland-Manager
        };

        /**
         * @class XWaylandManager
         * @brief Manager-Klasse für XWayland-Integration
         *
         * Diese Klasse ist für die Integration von XWayland in den Wayland-Compositor verantwortlich.
         * Sie startet und verwaltet den XWayland-Prozess.
         */
        class XWaylandManager : public QObject {
            Q_OBJECT
        public:
            /**
             * @brief Konstruktor
             * @param parent Elternobjekt
             */
            explicit XWaylandManager(QObject* parent = nullptr);

            /**
             * @brief Destruktor
             */
            ~XWaylandManager();

            /**
             * @brief Initialisiert den XWayland-Manager
             * @return true bei Erfolg, false bei Fehler
             */
            bool initialize();

            /**
             * @brief Prüft, ob XWayland läuft
             * @return true, wenn XWayland läuft, sonst false
             */
            bool isRunning() const;

            /**
             * @brief Beendet XWayland
             */
            void shutdown();

        signals:
            /**
             * @brief Signal, das ausgelöst wird, wenn XWayland gestartet wird
             */
            void started();

            /**
             * @brief Signal, das ausgelöst wird, wenn XWayland beendet wird
             */
            void stopped();

            /**
             * @brief Signal, das ausgelöst wird, wenn ein Fehler auftritt
             * @param errorMessage Fehlermeldung
             */
            void error(const QString& errorMessage);

        private:
            QProcess* m_xwaylandProcess; ///< XWayland-Prozess
            bool m_isRunning; ///< Flag, ob XWayland läuft
            QWaylandCompositor* m_compositor; ///< Wayland-Compositor
        };

        /**
         * @class LayerShellManager
         * @brief Manager-Klasse für das Layer-Shell-Protokoll
         *
         * Diese Klasse implementiert das Layer-Shell-Protokoll für den Wayland-Compositor.
         * Layer-Shell ermöglicht es Clients, Fenster in verschiedenen Ebenen anzuordnen.
         */
        class LayerShellManager : public QObject {
            Q_OBJECT
        public:
            /**
             * @brief Konstruktor
             * @param compositor Wayland-Compositor
             * @param parent Elternobjekt
             */
            explicit LayerShellManager(QWaylandCompositor* compositor, QObject* parent = nullptr);

            /**
             * @brief Destruktor
             */
            ~LayerShellManager();

            /**
             * @brief Initialisiert den Layer-Shell-Manager
             * @return true bei Erfolg, false bei Fehler
             */
            bool initialize();

            /**
             * @brief Definiert die verfügbaren Ebenen
             */
            enum Layer {
                BackgroundLayer = 0, ///< Hintergrundebene (unterste Ebene)
                BottomLayer = 1,     ///< Untere Ebene (über Hintergrund)
                TopLayer = 2,        ///< Obere Ebene (über normalen Fenstern)
                OverlayLayer = 3     ///< Overlay-Ebene (oberste Ebene)
            };

            /**
             * @brief Setzt die Ebene für eine Oberfläche
             * @param surface Wayland-Oberfläche
             * @param layer Ebene
             */
            void setLayerForSurface(QWaylandSurface* surface, Layer layer);

            /**
             * @brief Gibt die Ebene für eine Oberfläche zurück
             * @param surface Wayland-Oberfläche
             * @return Ebene
             */
            Layer getLayerForSurface(QWaylandSurface* surface) const;

        signals:
            /**
             * @brief Signal, das ausgelöst wird, wenn eine Oberfläche in einer Ebene erstellt wird
             * @param surface Wayland-Oberfläche
             * @param layer Ebene
             */
            void layerSurfaceCreated(QWaylandSurface* surface, Layer layer);

            /**
             * @brief Signal, das ausgelöst wird, wenn eine Oberfläche in einer Ebene zerstört wird
             * @param surface Wayland-Oberfläche
             */
            void layerSurfaceDestroyed(QWaylandSurface* surface);

        private:
            QWaylandCompositor* m_compositor; ///< Wayland-Compositor
            QMap<QWaylandSurface*, Layer> m_surfaceLayers; ///< Zuordnung von Oberflächen zu Ebenen
        };

        /**
         * @class PresentationTimeManager
         * @brief Manager-Klasse für das Presentation-Time-Protokoll
         *
         * Diese Klasse implementiert das Presentation-Time-Protokoll für den Wayland-Compositor.
         * Dieses Protokoll ermöglicht es Clients, Informationen über die Anzeigezeit von Frames zu erhalten.
         */
        class PresentationTimeManager : public QObject {
            Q_OBJECT
        public:
            /**
             * @brief Konstruktor
             * @param compositor Wayland-Compositor
             * @param parent Elternobjekt
             */
            explicit PresentationTimeManager(QWaylandCompositor* compositor, QObject* parent = nullptr);

            /**
             * @brief Destruktor
             */
            ~PresentationTimeManager();

            /**
             * @brief Initialisiert den Presentation-Time-Manager
             * @return true bei Erfolg, false bei Fehler
             */
            bool initialize();

            /**
             * @brief Meldet ein Frame-Präsentationsereignis
             * @param surface Wayland-Oberfläche
             * @param presentationTime Präsentationszeit in ns seit System-Start
             */
            void framePresentedEvent(QWaylandSurface* surface, quint64 presentationTime);

        signals:
            /**
             * @brief Signal, das ausgelöst wird, wenn eine Oberfläche Presentation-Time unterstützt
             * @param surface Wayland-Oberfläche
             */
            void presentationTimeSupported(QWaylandSurface* surface);

        private:
            QWaylandCompositor* m_compositor; ///< Wayland-Compositor
            QMap<QWaylandSurface*, bool> m_surfaceSupport; ///< Zuordnung von Oberflächen zu Unterstützungsstatus
        };

        /**
         * @class ViewporterManager
         * @brief Manager-Klasse für das Viewporter-Protokoll
         *
         * Diese Klasse implementiert das Viewporter-Protokoll für den Wayland-Compositor.
         * Dieses Protokoll ermöglicht es Clients, die Skalierung und Beschneidung von Oberflächen zu steuern.
         */
        class ViewporterManager : public QObject {
            Q_OBJECT
        public:
            /**
             * @brief Konstruktor
             * @param compositor Wayland-Compositor
             * @param parent Elternobjekt
             */
            explicit ViewporterManager(QWaylandCompositor* compositor, QObject* parent = nullptr);

            /**
             * @brief Destruktor
             */
            ~ViewporterManager();

            /**
             * @brief Initialisiert den Viewporter-Manager
             * @return true bei Erfolg, false bei Fehler
             */
            bool initialize();

            /**
             * @brief Setzt die Zielgröße für eine Oberfläche
             * @param surface Wayland-Oberfläche
             * @param destinationSize Zielgröße
             */
            void setViewportDestination(QWaylandSurface* surface, const QSize& destinationSize);

            /**
             * @brief Gibt die Zielgröße für eine Oberfläche zurück
             * @param surface Wayland-Oberfläche
             * @return Zielgröße
             */
            QSize getViewportDestination(QWaylandSurface* surface) const;

        signals:
            /**
             * @brief Signal, das ausgelöst wird, wenn sich der Viewport einer Oberfläche ändert
             * @param surface Wayland-Oberfläche
             * @param size Neue Größe
             */
            void viewportChanged(QWaylandSurface* surface, const QSize& size);

        private:
            QWaylandCompositor* m_compositor; ///< Wayland-Compositor
            QMap<QWaylandSurface*, QSize> m_surfaceViewports; ///< Zuordnung von Oberflächen zu Viewports
        };

    } // namespace Wayland
} // namespace VivoX
