/**
 * @file XWaylandIntegration.h
 * @brief Implementierung der XWayland-Integration
 *
 * Diese Datei enthält die Implementierung der XWayland-Integration, die für die
 * Unterstützung von X11-Anwendungen im Wayland-Compositor verantwortlich ist.
 */

#pragma once

#include <QObject>
#include <QWaylandCompositor>
#include <QWaylandSurface>
#include <QProcess>
#include <QMap>
#include <QTimer>
#include <QLocalSocket>
#include <QSocketNotifier>

// Vorwärtsdeklarationen für XCB-Typen
typedef struct xcb_connection_t xcb_connection_t;
typedef uint32_t xcb_window_t;
typedef struct xcb_create_notify_event_t xcb_create_notify_event_t;
typedef struct xcb_destroy_notify_event_t xcb_destroy_notify_event_t;
typedef struct xcb_map_request_event_t xcb_map_request_event_t;
typedef struct xcb_configure_request_event_t xcb_configure_request_event_t;

class QWaylandXdgSurface;

namespace VivoX {
    namespace Wayland {

        /**
         * @class XWaylandIntegration
         * @brief Implementierung der XWayland-Integration
         *
         * Diese Klasse implementiert die XWayland-Integration und ist für die
         * Verwaltung des XWayland-Servers und die Integration von X11-Anwendungen
         * in den Wayland-Compositor verantwortlich.
         */
        class XWaylandIntegration : public QObject {
            Q_OBJECT
            Q_PROPERTY(bool running READ isRunning NOTIFY runningChanged)

        public:
            /**
             * @brief Konstruktor
             * @param compositor Wayland-Compositor
             * @param parent Elternobjekt
             */
            explicit XWaylandIntegration(QWaylandCompositor* compositor, QObject* parent = nullptr);

            /**
             * @brief Destruktor
             */
            ~XWaylandIntegration();

            /**
             * @brief Startet den XWayland-Server
             * @return true bei Erfolg, false bei Fehler
             */
            bool start();

            /**
             * @brief Stoppt den XWayland-Server
             */
            void stop();

            /**
             * @brief Prüft, ob der XWayland-Server läuft
             * @return true, wenn der XWayland-Server läuft, sonst false
             */
            bool isRunning() const;

            /**
             * @brief Gibt den Wayland-Compositor zurück
             * @return Wayland-Compositor
             */
            QWaylandCompositor* compositor() const;

        signals:
            /**
             * @brief Signal, das ausgelöst wird, wenn sich der Status des XWayland-Servers ändert
             * @param running true, wenn der XWayland-Server läuft, sonst false
             */
            void runningChanged(bool running);

            /**
             * @brief Signal, das ausgelöst wird, wenn eine neue X11-Oberfläche erstellt wird
             * @param surface Wayland-Oberfläche für die X11-Anwendung
             * @param windowId X11-Fenster-ID
             */
            void x11SurfaceCreated(QWaylandSurface* surface, uint32_t windowId);

            /**
             * @brief Signal, das ausgelöst wird, wenn eine X11-Oberfläche zerstört wird
             * @param surface Wayland-Oberfläche für die X11-Anwendung
             * @param windowId X11-Fenster-ID
             */
            void x11SurfaceDestroyed(QWaylandSurface* surface, uint32_t windowId);

        private slots:
            /**
             * @brief Wird aufgerufen, wenn der XWayland-Server beendet wird
             * @param exitCode Exit-Code des XWayland-Servers
             * @param exitStatus Exit-Status des XWayland-Servers
             */
            void handleXWaylandFinished(int exitCode, QProcess::ExitStatus exitStatus);

            /**
             * @brief Wird aufgerufen, wenn eine neue Oberfläche erstellt wird
             * @param surface Neue Oberfläche
             */
            void handleSurfaceCreated(QWaylandSurface* surface);

            /**
             * @brief Verarbeitet XCB-Events vom X-Server
             */
            void handleXcbEvents();

            /**
             * @brief Wird aufgerufen, wenn der WM-Socket verbunden ist
             */
            void handleWmSocketConnected();

            /**
             * @brief Wird aufgerufen, wenn ein WM-Socket-Fehler auftritt
             * @param error Fehlertyp
             */
            void handleWmSocketError(QLocalSocket::LocalSocketError error);

        private:
            QWaylandCompositor* m_compositor; ///< Wayland-Compositor
            QProcess* m_xwaylandProcess; ///< XWayland-Prozess
            bool m_running; ///< Gibt an, ob der XWayland-Server läuft
            QMap<QWaylandSurface*, uint32_t> m_x11Surfaces; ///< Zuordnung von Wayland-Oberflächen zu X11-Fenster-IDs

            xcb_connection_t* m_xcbConnection; ///< XCB-Verbindung zum X-Server
            int m_wmFd; ///< File-Descriptor für den Window-Manager-Socket
            int m_displayFd; ///< File-Descriptor für den Display-Socket
            int m_displayNumber; ///< X-Display-Nummer
            int m_restartCount; ///< Anzahl der Neustart-Versuche
            QTimer* m_restartTimer; ///< Timer für automatischen Neustart
            QLocalSocket* m_wmSocket; ///< Socket für Kommunikation mit dem Window-Manager
            QMap<QString, xcb_window_t> m_atoms; ///< X11-Atome für Fenstermanagement

            /**
             * @brief Findet den Pfad zum XWayland-Binary
             * @return Pfad zum XWayland-Binary
             */
            QString findXWaylandBinary() const;

            /**
             * @brief Erstellt die Umgebungsvariablen für den XWayland-Server
             * @return Umgebungsvariablen für den XWayland-Server
             */
            QProcessEnvironment createXWaylandEnvironment() const;

            /**
             * @brief Erstellt die Kommandozeilenargumente für den XWayland-Server
             * @return Kommandozeilenargumente für den XWayland-Server
             */
            QStringList createXWaylandArguments() const;

            /**
             * @brief Erstellt Socket-Paare für die Kommunikation mit XWayland
             * @return true bei Erfolg, false bei Fehler
             */
            bool createSocketPairs();

            /**
             * @brief Richtet den Display-Socket ein
             * @return true bei Erfolg, false bei Fehler
             */
            bool setupDisplaySocket();

            /**
             * @brief Wartet auf die Display-Nummer vom XWayland-Server
             * @return true bei Erfolg, false bei Fehler
             */
            bool waitForDisplayNumber();

            /**
             * @brief Verbindet zum X-Server
             * @return true bei Erfolg, false bei Fehler
             */
            bool connectToXServer();

            /**
             * @brief Richtet das XCB-Event-Handling ein
             */
            void setupXcbEventHandling();

            /**
             * @brief Richtet den Window-Manager ein
             */
            void setupWindowManager();

            /**
             * @brief Initialisiert X11-Atome für Fenstermanagement
             */
            void initializeAtoms();

            /**
             * @brief Bereinigt Ressourcen nach einem Fehler
             */
            void cleanupAfterFailure();

            /**
             * @brief Generiert eine temporäre Xauth-Datei
             * @return Pfad zur generierten Xauth-Datei
             */
            QString generateXauthFile() const;

            /**
             * @brief Verarbeitet X11 Create-Notify-Events
             * @param event X11 Create-Notify-Event
             */
            void handleXcbCreateNotify(xcb_create_notify_event_t* event);

            /**
             * @brief Verarbeitet X11 Destroy-Notify-Events
             * @param event X11 Destroy-Notify-Event
             */
            void handleXcbDestroyNotify(xcb_destroy_notify_event_t* event);

            /**
             * @brief Verarbeitet X11 Map-Request-Events
             * @param event X11 Map-Request-Event
             */
            void handleXcbMapRequest(xcb_map_request_event_t* event);

            /**
             * @brief Verarbeitet X11 Configure-Request-Events
             * @param event X11 Configure-Request-Event
             */
            void handleXcbConfigureRequest(xcb_configure_request_event_t* event);

            /**
             * @brief Richtet eine X11-Oberfläche für die Integration ein
             * @param surface Wayland-Oberfläche
             * @param windowId X11-Fenster-ID
             */
            void setupX11Surface(QWaylandSurface* surface, uint32_t windowId);

            /**
             * @brief Findet die XDG-Oberfläche für eine Wayland-Oberfläche
             * @param surface Wayland-Oberfläche
             * @return XDG-Oberfläche oder nullptr, wenn keine existiert
             */
            QWaylandXdgSurface* findXdgSurface(QWaylandSurface* surface);
        };

    } // namespace Wayland
} // namespace VivoX
