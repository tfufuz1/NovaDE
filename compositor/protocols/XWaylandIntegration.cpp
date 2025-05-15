/**
 * @file XWaylandIntegration.cpp
 * @brief Implementierung der XWayland-Integration
 * 
 * Diese Datei enthält die Implementierung der XWayland-Integration, die für die
 * Unterstützung von X11-Anwendungen im Wayland-Compositor verantwortlich ist.
 */

 #include "XWaylandIntegration.h"
 #include "../core/Logger.h"
 
 #include <QStandardPaths>
 #include <QDir>
 #include <QFile>
 #include <QSocketNotifier>
 #include <QCoreApplication>
 #include <QRandomGenerator>
 #include <QWaylandXdgSurface>
 #include <QWaylandClient>
 
 // X11-spezifische Includes
 #include <xcb/xcb.h>
 #include <xcb/composite.h>
 #include <xcb/xproto.h>
 
 namespace VivoX {
 namespace Wayland {
 
 // Konstanten für XWayland
 constexpr int X_DISPLAY_NUMBER = 0;
 constexpr int MAX_RESTART_ATTEMPTS = 3;
 constexpr int RESTART_DELAY_MS = 1000;
 
 XWaylandIntegration::XWaylandIntegration(QWaylandCompositor* compositor, QObject* parent)
     : QObject(parent)
     , m_compositor(compositor)
     , m_xwaylandProcess(nullptr)
     , m_running(false)
     , m_x11Surfaces()
     , m_xcbConnection(nullptr)
     , m_wmFd(-1)
     , m_displayFd(-1)
     , m_displayNumber(X_DISPLAY_NUMBER)
     , m_restartCount(0)
     , m_restartTimer(new QTimer(this))
     , m_wmSocket(new QLocalSocket(this))
 {
     Core::Logger::instance().info("XWaylandIntegration erstellt", "Wayland");
     
     // Verbinde Signale des Compositors
     connect(m_compositor, &QWaylandCompositor::surfaceCreated,
             this, &XWaylandIntegration::handleSurfaceCreated);
             
     // Verbinde Restart-Timer
     m_restartTimer->setSingleShot(true);
     connect(m_restartTimer, &QTimer::timeout, this, [this]() {
         if (m_restartCount < MAX_RESTART_ATTEMPTS) {
             Core::Logger::instance().info(
                 QString("Versuche XWayland-Server neu zu starten (Versuch %1 von %2)")
                     .arg(m_restartCount + 1)
                     .arg(MAX_RESTART_ATTEMPTS),
                 "Wayland"
             );
             start();
         } else {
             Core::Logger::instance().error(
                 "Maximale Anzahl an Neustartversuchen erreicht. XWayland-Integration deaktiviert.",
                 "Wayland"
             );
         }
     });
     
     // Verbinde WM-Socket-Signale
     connect(m_wmSocket, &QLocalSocket::connected, this, &XWaylandIntegration::handleWmSocketConnected);
     connect(m_wmSocket, &QLocalSocket::errorOccurred, this, &XWaylandIntegration::handleWmSocketError);
 }
 
 XWaylandIntegration::~XWaylandIntegration()
 {
     // Stoppe den XWayland-Server, falls er noch läuft
     if (m_running) {
         stop();
     }
     
     // Schließe XCB-Verbindung
     if (m_xcbConnection) {
         xcb_disconnect(m_xcbConnection);
         m_xcbConnection = nullptr;
     }
     
     // Schließe Sockets
     if (m_wmFd >= 0) {
         close(m_wmFd);
         m_wmFd = -1;
     }
     
     if (m_displayFd >= 0) {
         close(m_displayFd);
         m_displayFd = -1;
     }
     
     Core::Logger::instance().info("XWaylandIntegration zerstört", "Wayland");
 }
 
 bool XWaylandIntegration::start()
 {
     if (m_running) {
         Core::Logger::instance().warning("XWayland-Server läuft bereits", "Wayland");
         return true;
     }
     
     // Finde den Pfad zum XWayland-Binary
     QString xwaylandBinary = findXWaylandBinary();
     if (xwaylandBinary.isEmpty()) {
         Core::Logger::instance().error("XWayland-Binary nicht gefunden", "Wayland");
         return false;
     }
     
     // Erstelle Socket-Paare für die Kommunikation mit XWayland
     if (!createSocketPairs()) {
         Core::Logger::instance().error("Konnte Socket-Paare nicht erstellen", "Wayland");
         return false;
     }
     
     // Bestimme die Display-Nummer
     if (!setupDisplaySocket()) {
         Core::Logger::instance().error("Konnte Display-Socket nicht erstellen", "Wayland");
         return false;
     }
     
     // Erstelle den XWayland-Prozess
     m_xwaylandProcess = new QProcess(this);
     
     // Setze die Umgebungsvariablen
     m_xwaylandProcess->setProcessEnvironment(createXWaylandEnvironment());
     
     // Verbinde Signale
     connect(m_xwaylandProcess, QOverload<int, QProcess::ExitStatus>::of(&QProcess::finished),
             this, &XWaylandIntegration::handleXWaylandFinished);
     
     connect(m_xwaylandProcess, &QProcess::errorOccurred, this, [this](QProcess::ProcessError error) {
         Core::Logger::instance().error(
             QString("XWayland-Prozessfehler: %1").arg(m_xwaylandProcess->errorString()),
             "Wayland"
         );
     });
     
     // Verbinde StdErr für Logging
     connect(m_xwaylandProcess, &QProcess::readyReadStandardError, this, [this]() {
         QString errorOutput = QString::fromUtf8(m_xwaylandProcess->readAllStandardError()).trimmed();
         if (!errorOutput.isEmpty()) {
             Core::Logger::instance().warning(
                 QString("XWayland stderr: %1").arg(errorOutput),
                 "Wayland"
             );
         }
     });
     
     // Starte den XWayland-Server
     m_xwaylandProcess->start(xwaylandBinary, createXWaylandArguments());
     
     if (!m_xwaylandProcess->waitForStarted(5000)) {
         Core::Logger::instance().error(
             QString("Fehler beim Starten des XWayland-Servers: %1")
                 .arg(m_xwaylandProcess->errorString()),
             "Wayland"
         );
         cleanupAfterFailure();
         return false;
     }
     
     // Warte auf Ausgabe der Display-Nummer
     if (!waitForDisplayNumber()) {
         Core::Logger::instance().error("Timeout beim Warten auf Display-Nummer", "Wayland");
         cleanupAfterFailure();
         return false;
     }
     
     // Verbinde zum X-Server
     if (!connectToXServer()) {
         Core::Logger::instance().error("Konnte nicht zum X-Server verbinden", "Wayland");
         cleanupAfterFailure();
         return false;
     }
     
     // Verbinde zum Window-Manager-Socket
     m_wmSocket->connectToServer(QString("X%1").arg(m_displayNumber));
     
     m_running = true;
     m_restartCount = 0; // Zurücksetzen des Zählers bei erfolgreichen Start
     emit runningChanged(true);
     
     Core::Logger::instance().info(
         QString("XWayland-Server gestartet auf Display :%1").arg(m_displayNumber),
         "Wayland"
     );
     return true;
 }
 
 void XWaylandIntegration::stop()
 {
     if (!m_running) {
         Core::Logger::instance().warning("XWayland-Server läuft nicht", "Wayland");
         return;
     }
     
     // Beende den XWayland-Server
     if (m_xwaylandProcess) {
         m_xwaylandProcess->terminate();
         if (!m_xwaylandProcess->waitForFinished(5000)) {
             Core::Logger::instance().warning("XWayland-Server reagiert nicht, wird gekillt", "Wayland");
             m_xwaylandProcess->kill();
         }
         
         delete m_xwaylandProcess;
         m_xwaylandProcess = nullptr;
     }
     
     // Bereinige XCB-Verbindung
     if (m_xcbConnection) {
         xcb_disconnect(m_xcbConnection);
         m_xcbConnection = nullptr;
     }
     
     // Schließe Sockets
     if (m_wmFd >= 0) {
         close(m_wmFd);
         m_wmFd = -1;
     }
     
     if (m_displayFd >= 0) {
         close(m_displayFd);
         m_displayFd = -1;
     }
     
     m_running = false;
     emit runningChanged(false);
     
     Core::Logger::instance().info("XWayland-Server gestoppt", "Wayland");
 }
 
 bool XWaylandIntegration::isRunning() const
 {
     return m_running;
 }
 
 QWaylandCompositor* XWaylandIntegration::compositor() const
 {
     return m_compositor;
 }
 
 void XWaylandIntegration::handleXWaylandFinished(int exitCode, QProcess::ExitStatus exitStatus)
 {
     Core::Logger::instance().info(
         QString("XWayland-Server beendet: ExitCode=%1, ExitStatus=%2")
             .arg(exitCode)
             .arg(exitStatus == QProcess::NormalExit ? "Normal" : "Crash"),
         "Wayland"
     );
     
     // Bereinige Ressourcen
     if (m_xwaylandProcess) {
         delete m_xwaylandProcess;
         m_xwaylandProcess = nullptr;
     }
     
     // Schließe XCB-Verbindung
     if (m_xcbConnection) {
         xcb_disconnect(m_xcbConnection);
         m_xcbConnection = nullptr;
     }
     
     // Schließe Sockets
     if (m_wmFd >= 0) {
         close(m_wmFd);
         m_wmFd = -1;
     }
     
     if (m_displayFd >= 0) {
         close(m_displayFd);
         m_displayFd = -1;
     }
     
     m_running = false;
     emit runningChanged(false);
     
     // Entferne alle X11-Oberflächen
     QList<QWaylandSurface*> surfaces = m_x11Surfaces.keys();
     for (QWaylandSurface* surface : surfaces) {
         uint32_t windowId = m_x11Surfaces.value(surface);
         m_x11Surfaces.remove(surface);
         emit x11SurfaceDestroyed(surface, windowId);
     }
     
     // Versuche automatisch neu zu starten, wenn der Server unerwartet beendet wurde
     if (exitStatus == QProcess::CrashExit || (exitStatus == QProcess::NormalExit && exitCode != 0)) {
         m_restartCount++;
         m_restartTimer->start(RESTART_DELAY_MS);
     }
 }
 void XWaylandIntegration::handleSurfaceCreated(QWaylandSurface* surface)
 {
     if (!m_running || !m_xcbConnection) {
         return;
     }
     
     // Hole den Client für die Oberfläche
     QWaylandClient* client = surface->client();
     if (!client) {
         return;
     }
     
     // Prüfe, ob es sich um eine X11-Oberfläche handelt
     // X11-Oberflächen haben spezifische Eigenschaften im Wayland-Client
     // Wir können dies über spezielle Properties in der Surface erkennen
     
     // Prüfe zunächst nach X11-spezifischer Property
     QWaylandResource* surfaceResource = surface->resource();
     if (!surfaceResource) {
         return;
     }
     
     // Der X11-Window-Identifier wird als Property der Oberfläche gesetzt
     uint32_t windowId = 0;
     
     // XDG-Surface-Protokoll verwenden, um an weitere Informationen zu kommen
     QWaylandXdgSurface* xdgSurface = findXdgSurface(surface);
     if (!xdgSurface) {
         return;
     }
     
     // Analysiere den Oberflächentyp und die Eigenschaften
     // In einer wirklichen Implementierung würden wir hier nach X11-spezifischen
     // Eigenschaften suchen, die durch XWayland gesetzt werden
     
     // Suche in der XCB-Verbindung nach allen Top-Level-Windows
     xcb_query_tree_cookie_t cookie = xcb_query_tree(m_xcbConnection, XCB_WINDOW_NONE);
     xcb_query_tree_reply_t* reply = xcb_query_tree_reply(m_xcbConnection, cookie, nullptr);
     
     if (!reply) {
         return;
     }
     
     // Anzahl der Kinder des Root-Fensters
     uint32_t childrenCount = xcb_query_tree_children_length(reply);
     xcb_window_t* children = xcb_query_tree_children(reply);
     
     // Durchsuche alle Top-Level-Fenster nach einer passenden Oberfläche
     for (uint32_t i = 0; i < childrenCount; i++) {
         xcb_window_t window = children[i];
         
         // Prüfe für jedes Fenster, ob es zu dieser Wayland-Oberfläche gehört
         // Wir verwenden hier verschiedene Heuristiken:
         // 1. PID-Vergleich
         // 2. Titel-Vergleich
         // 3. Geometrie-Vergleich
         
         // Get window properties
         xcb_get_property_cookie_t pidCookie = xcb_get_property(
             m_xcbConnection,
             0,
             window,
             XCB_ATOM_WM_PID,
             XCB_ATOM_CARDINAL,
             0,
             1
         );
         
         xcb_get_property_reply_t* pidReply = xcb_get_property_reply(
             m_xcbConnection,
             pidCookie,
             nullptr
         );
         
         bool matchFound = false;
         
         if (pidReply && pidReply->type == XCB_ATOM_CARDINAL && pidReply->format == 32) {
             uint32_t* pid = static_cast<uint32_t*>(xcb_get_property_value(pidReply));
             
             // Vergleiche PID mit dem Client-PID
             if (pid && *pid == client->processId()) {
                 matchFound = true;
                 windowId = window;
             }
             
             free(pidReply);
         }
         
         if (!matchFound) {
             // Weitere Heuristiken anwenden (Titel, Größe, etc.)
             // ...
             
             // Wenn wir mit hoher Wahrscheinlichkeit ein Match haben:
             // matchFound = true;
             // windowId = window;
         }
         
         if (matchFound) {
             // Wir haben ein passendes X11-Fenster für diese Wayland-Oberfläche gefunden
             break;
         }
     }
     
     free(reply);
     
     // Wenn wir eine WindowID gefunden haben, markieren wir diese Oberfläche als X11-Oberfläche
     if (windowId != 0) {
         // Füge die X11-Oberfläche zur Liste hinzu
         m_x11Surfaces.insert(surface, windowId);
         
         // Verbinde Signale
         connect(surface, &QWaylandSurface::destroyed, this, [this, surface]() {
             if (m_x11Surfaces.contains(surface)) {
                 uint32_t windowId = m_x11Surfaces.value(surface);
                 m_x11Surfaces.remove(surface);
                 emit x11SurfaceDestroyed(surface, windowId);
             }
         });
         
         // Weitere Oberflächeneigenschaften einrichten
         setupX11Surface(surface, windowId);
         
         // Sende Signal, dass eine neue X11-Oberfläche erstellt wurde
         emit x11SurfaceCreated(surface, windowId);
         
         Core::Logger::instance().info(
             QString("X11-Oberfläche erkannt: WindowID=0x%1")
                 .arg(windowId, 0, 16),
             "Wayland"
         );
     }
 }
 
 void XWaylandIntegration::setupX11Surface(QWaylandSurface* surface, uint32_t windowId)
 {
     // Konfiguriere die X11-Oberfläche mit zusätzlichen Eigenschaften
     
     // Hole Fenstereigenschaften vom X-Server
     xcb_get_property_cookie_t titleCookie = xcb_get_property(
         m_xcbConnection,
         0,
         windowId,
         XCB_ATOM_WM_NAME,
         XCB_ATOM_STRING,
         0,
         128
     );
     
     xcb_get_property_reply_t* titleReply = xcb_get_property_reply(
         m_xcbConnection,
         titleCookie,
         nullptr
     );
     
     if (titleReply && titleReply->type == XCB_ATOM_STRING) {
         char* title = static_cast<char*>(xcb_get_property_value(titleReply));
         int titleLength = xcb_get_property_value_length(titleReply);
         
         QString windowTitle = QString::fromUtf8(title, titleLength);
         
         // Setze den Titel auf die Wayland-Oberfläche (falls möglich)
         if (QWaylandXdgSurface* xdgSurface = findXdgSurface(surface)) {
             if (QWaylandXdgToplevel* toplevel = xdgSurface->toplevel()) {
                 // Hier könnten wir weitere Operationen für Top-Level-Fenster durchführen
                 // z.B. Maximieren, Minimieren, etc. basierend auf dem X11-Fensterzustand
             }
         }
         
         free(titleReply);
     }
     
     // Hole Geometrieinformationen
     xcb_get_geometry_cookie_t geomCookie = xcb_get_geometry(m_xcbConnection, windowId);
     xcb_get_geometry_reply_t* geomReply = xcb_get_geometry_reply(m_xcbConnection, geomCookie, nullptr);
     
     if (geomReply) {
         // Geometrie des X11-Fensters auf die Wayland-Oberfläche übertragen
         // Dies ist ein komplexer Prozess und erfordert eine tiefere Integration
         // mit dem Wayland-Compositor
         
         free(geomReply);
     }
 }
 
 QWaylandXdgSurface* XWaylandIntegration::findXdgSurface(QWaylandSurface* surface)
 {
     // Suche nach XDG-Surface für die gegebene Wayland-Oberfläche
     for (QWaylandXdgSurface* xdgSurface : surface->extensions<QWaylandXdgSurface>()) {
         return xdgSurface;
     }
     
     return nullptr;
 }
 
 QString XWaylandIntegration::findXWaylandBinary() const
 {
     // Suche nach dem XWayland-Binary in den Standard-Pfaden
     QString xwaylandPath = QStandardPaths::findExecutable("Xwayland");
     if (!xwaylandPath.isEmpty()) {
         return xwaylandPath;
     }
     
     // Suche nach dem XWayland-Binary in zusätzlichen Pfaden
     QStringList additionalPaths = {
         "/usr/bin/Xwayland",
         "/usr/local/bin/Xwayland",
         "/opt/X11/bin/Xwayland",
         "/usr/libexec/Xorg.bin", // Fedora/RHEL Pfad
         "/usr/lib/xorg/Xorg"     // Ubuntu/Debian Pfad
     };
     
     for (const QString& path : additionalPaths) {
         if (QFile::exists(path)) {
             return path;
         }
     }
     
     Core::Logger::instance().error("XWayland-Binary nicht gefunden", "Wayland");
     return QString();
 }
 
 QProcessEnvironment XWaylandIntegration::createXWaylandEnvironment() const
 {
     QProcessEnvironment env = QProcessEnvironment::systemEnvironment();
     
     // Setze die Wayland-Socket-Variable
     env.insert("WAYLAND_DISPLAY", m_compositor->socketName());
     
     // Setze die X11-Display-Variable
     env.insert("DISPLAY", QString(":%1").arg(m_displayNumber));
     
     // Setze weitere Umgebungsvariablen für XWayland
     env.insert("XAUTHORITY", generateXauthFile());
     
     return env;
 }
 
 QStringList XWaylandIntegration::createXWaylandArguments() const
 {
     QStringList args;
     
     // Grundlegende Argumente für XWayland
     args << "-rootless"
          << "-terminate"
          << QString("-displayfd") << QString::number(m_displayFd)
          << QString("-wm") << QString::number(m_wmFd)
          << QString(":%1").arg(m_displayNumber);
     
     // Weitere Argumente für bessere Leistung und Kompatibilität
     args << "-nolisten" << "tcp"
          << "-listencups"
          << "-auth" << generateXauthFile();
     
     return args;
 }
 
 bool XWaylandIntegration::createSocketPairs()
 {
     // Erstelle Socket-Paar für Window Manager
     int wmSockets[2];
     if (socketpair(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0, wmSockets) < 0) {
         Core::Logger::instance().error(
             QString("Konnte WM-Socket-Paar nicht erstellen: %1")
                 .arg(strerror(errno)),
             "Wayland"
         );
         return false;
     }
     
     m_wmFd = wmSockets[0];
     
     // Erstelle Pipe für Display-Nummer
     int displayPipe[2];
     if (pipe(displayPipe) < 0) {
         Core::Logger::instance().error(
             QString("Konnte Display-Pipe nicht erstellen: %1")
                 .arg(strerror(errno)),
             "Wayland"
         );
         close(wmSockets[0]);
         close(wmSockets[1]);
         return false;
     }
     
     m_displayFd = displayPipe[1];
     
     return true;
 }
 
 bool XWaylandIntegration::setupDisplaySocket()
 {
     // Suche nach einer freien Display-Nummer
     for (int i = 0; i < 32; i++) {
         QString lockFilePath = QString("/tmp/.X%1-lock").arg(i);
         QString socketPath = QString("/tmp/.X11-unix/X%1").arg(i);
         
         if (!QFile::exists(lockFilePath) && !QFile::exists(socketPath)) {
             m_displayNumber = i;
             return true;
         }
     }
     
     Core::Logger::instance().error("Konnte keine freie Display-Nummer finden", "Wayland");
     return false;
 }
 
 bool XWaylandIntegration::waitForDisplayNumber()
 {
     // Warte auf die Ausgabe der Display-Nummer von XWayland
     QFile file;
     if (!file.open(m_displayFd, QIODevice::ReadOnly)) {
         Core::Logger::instance().error("Konnte Display-Pipe nicht öffnen", "Wayland");
         return false;
     }
     
     // Lese die Display-Nummer
     QByteArray displayNumberStr = file.readLine();
     bool ok;
     int displayNumber = displayNumberStr.trimmed().toInt(&ok);
     
     if (!ok) {
         Core::Logger::instance().error("Ungültige Display-Nummer erhalten", "Wayland");
         return false;
     }
     
     m_displayNumber = displayNumber;
     return true;
 }
 
 bool XWaylandIntegration::connectToXServer()
 {
     // Warte einen Moment, bis der X-Server gestartet ist
     QThread::msleep(100);
     
     // Verbinde zum X-Server
     QString displayName = QString(":%1").arg(m_displayNumber);
     m_xcbConnection = xcb_connect(displayName.toUtf8().constData(), nullptr);
     
     if (xcb_connection_has_error(m_xcbConnection)) {
         Core::Logger::instance().error("Fehler bei der Verbindung zum X-Server", "Wayland");
         xcb_disconnect(m_xcbConnection);
         m_xcbConnection = nullptr;
         return false;
     }
     
     // Prüfe, ob Composite-Extension verfügbar ist
     const xcb_query_extension_reply_t *compositeExt = 
         xcb_get_extension_data(m_xcbConnection, &xcb_composite_id);
     
     if (!compositeExt || !compositeExt->present) {
         Core::Logger::instance().warning("Composite-Extension nicht verfügbar", "Wayland");
     }
     
     // Registriere für X11-Events
     setupXcbEventHandling();
     
     return true;
 }
 
 void XWaylandIntegration::setupXcbEventHandling()
 {
     // Hole den X11-Socket für Event-Handling
     int xcbFd = xcb_get_file_descriptor(m_xcbConnection);
     
     if (xcbFd >= 0) {
         // Erstelle QSocketNotifier für XCB-Events
         QSocketNotifier* notifier = new QSocketNotifier(xcbFd, QSocketNotifier::Read, this);
         connect(notifier, &QSocketNotifier::activated, this, &XWaylandIntegration::handleXcbEvents);
     }
 }
 
 void XWaylandIntegration::handleXcbEvents()
 {
     if (!m_xcbConnection) {
         return;
     }
     
     // Verarbeite alle ausstehenden XCB-Events
     xcb_generic_event_t* event;
     while ((event = xcb_poll_for_event(m_xcbConnection))) {
         uint8_t responseType = event->response_type & ~0x80;
         
         switch (responseType) {
             case XCB_CREATE_NOTIFY: {
                 xcb_create_notify_event_t* createEvent = 
                     reinterpret_cast<xcb_create_notify_event_t*>(event);
                 handleXcbCreateNotify(createEvent);
                 break;
             }
             case XCB_DESTROY_NOTIFY: {
                 xcb_destroy_notify_event_t* destroyEvent = 
                     reinterpret_cast<xcb_destroy_notify_event_t*>(event);
                 handleXcbDestroyNotify(destroyEvent);
                 break;
             }
             case XCB_CONFIGURE_REQUEST: {
                 xcb_configure_request_event_t* configureEvent = 
                     reinterpret_cast<xcb_configure_request_event_t*>(event);
                 handleXcbConfigureRequest(configureEvent);
                 break;
             }
             case XCB_MAP_REQUEST: {
                 xcb_map_request_event_t* mapEvent = 
                     reinterpret_cast<xcb_map_request_event_t*>(event);
                 handleXcbMapRequest(mapEvent);
                 break;
             }
             // Weitere Event-Handler hier
         }
         
         free(event);
     }
     
     // Prüfe auf Verbindungsfehler
     if (xcb_connection_has_error(m_xcbConnection)) {
         Core::Logger::instance().error("XCB-Verbindungsfehler aufgetreten", "Wayland");
         stop();
     }
 }
 
 void XWaylandIntegration::handleXcbCreateNotify(xcb_create_notify_event_t* event)
 {
     Core::Logger::instance().debug(
         QString("X11 CreateNotify: window=0x%1 parent=0x%2")
             .arg(event->window, 0, 16)
             .arg(event->parent, 0, 16),
         "Wayland"
     );
     
     // Hier könnten wir neue Windows registrieren
 }
 
 void XWaylandIntegration::handleXcbDestroyNotify(xcb_destroy_notify_event_t* event)
 {
     Core::Logger::instance().debug(
         QString("X11 DestroyNotify: window=0x%1")
             .arg(event->window, 0, 16),
         "Wayland"
     );
     
     // Hier könnten wir die Zerstörung von Windows verarbeiten
     for (auto it = m_x11Surfaces.begin(); it != m_x11Surfaces.end(); ) {
         if (it.value() == event->window) {
             QWaylandSurface* surface = it.key();
             uint32_t windowId = it.value();
             
             it = m_x11Surfaces.erase(it);
             emit x11SurfaceDestroyed(surface, windowId);
         } else {
             ++it;
            }
        }
    }
    
    void XWaylandIntegration::handleXcbMapRequest(xcb_map_request_event_t* event)
    {
        Core::Logger::instance().debug(
            QString("X11 MapRequest: window=0x%1")
                .arg(event->window, 0, 16),
            "Wayland"
        );
        
        // Hole Fenstereigenschaften
        xcb_get_window_attributes_cookie_t attrCookie = 
            xcb_get_window_attributes(m_xcbConnection, event->window);
        xcb_get_window_attributes_reply_t* attrReply = 
            xcb_get_window_attributes_reply(m_xcbConnection, attrCookie, nullptr);
        
        if (!attrReply) {
            return;
        }
        
        // Prüfe, ob das Fenster sichtbar sein soll
        if (!(attrReply->override_redirect)) {
            // Mappe das Fenster (erlaube dem Fenster, sichtbar zu werden)
            xcb_map_window(m_xcbConnection, event->window);
        }
        
        free(attrReply);
    }
    
    void XWaylandIntegration::handleXcbConfigureRequest(xcb_configure_request_event_t* event)
    {
        Core::Logger::instance().debug(
            QString("X11 ConfigureRequest: window=0x%1 x=%2 y=%3 width=%4 height=%5")
                .arg(event->window, 0, 16)
                .arg(event->x)
                .arg(event->y)
                .arg(event->width)
                .arg(event->height),
            "Wayland"
        );
        
        // Erstelle die Konfigurationsänderung
        uint16_t mask = 0;
        uint32_t values[7];
        int i = 0;
        
        if (event->value_mask & XCB_CONFIG_WINDOW_X) {
            mask |= XCB_CONFIG_WINDOW_X;
            values[i++] = event->x;
        }
        
        if (event->value_mask & XCB_CONFIG_WINDOW_Y) {
            mask |= XCB_CONFIG_WINDOW_Y;
            values[i++] = event->y;
        }
        
        if (event->value_mask & XCB_CONFIG_WINDOW_WIDTH) {
            mask |= XCB_CONFIG_WINDOW_WIDTH;
            values[i++] = event->width;
        }
        
        if (event->value_mask & XCB_CONFIG_WINDOW_HEIGHT) {
            mask |= XCB_CONFIG_WINDOW_HEIGHT;
            values[i++] = event->height;
        }
        
        if (event->value_mask & XCB_CONFIG_WINDOW_BORDER_WIDTH) {
            mask |= XCB_CONFIG_WINDOW_BORDER_WIDTH;
            values[i++] = event->border_width;
        }
        
        if (event->value_mask & XCB_CONFIG_WINDOW_SIBLING) {
            mask |= XCB_CONFIG_WINDOW_SIBLING;
            values[i++] = event->sibling;
        }
        
        if (event->value_mask & XCB_CONFIG_WINDOW_STACK_MODE) {
            mask |= XCB_CONFIG_WINDOW_STACK_MODE;
            values[i++] = event->stack_mode;
        }
        
        // Wende die Konfiguration an
        xcb_configure_window(m_xcbConnection, event->window, mask, values);
        xcb_flush(m_xcbConnection);
        
        // Aktualisiere die Wayland-Oberfläche, falls eine zugeordnet ist
        for (auto it = m_x11Surfaces.begin(); it != m_x11Surfaces.end(); ++it) {
            if (it.value() == event->window) {
                QWaylandSurface* surface = it.key();
                if (surface) {
                    // Aktualisiere die Geometrie der Wayland-Oberfläche
                    if (event->value_mask & (XCB_CONFIG_WINDOW_WIDTH | XCB_CONFIG_WINDOW_HEIGHT)) {
                        // In einer vollständigen Implementierung würden wir hier die Größe der
                        // Wayland-Oberfläche aktualisieren
                    }
                }
                break;
            }
        }
    }
    
    void XWaylandIntegration::handleWmSocketConnected()
    {
        Core::Logger::instance().info("Verbunden mit XWayland Window Manager Socket", "Wayland");
        
        // Initialisiere Window-Manager-Funktionalität
        setupWindowManager();
    }
    
    void XWaylandIntegration::handleWmSocketError(QLocalSocket::LocalSocketError error)
    {
        Core::Logger::instance().error(
            QString("WM-Socket-Fehler: %1").arg(m_wmSocket->errorString()),
            "Wayland"
        );
    }
    
    void XWaylandIntegration::setupWindowManager()
    {
        if (!m_xcbConnection) {
            Core::Logger::instance().error("Keine XCB-Verbindung verfügbar", "Wayland");
            return;
        }
        
        // Hole den Root-Screen
        xcb_screen_t* screen = xcb_setup_roots_iterator(xcb_get_setup(m_xcbConnection)).data;
        if (!screen) {
            Core::Logger::instance().error("Konnte keinen XCB-Screen finden", "Wayland");
            return;
        }
        
        // Registriere für die SubstructureRedirect auf dem Root-Fenster
        uint32_t values = XCB_EVENT_MASK_SUBSTRUCTURE_NOTIFY | 
                          XCB_EVENT_MASK_SUBSTRUCTURE_REDIRECT;
        
        xcb_change_window_attributes(
            m_xcbConnection,
            screen->root,
            XCB_CW_EVENT_MASK,
            &values
        );
        
        xcb_flush(m_xcbConnection);
        
        // Erhalte X-Atome für Fenstermanagement
        initializeAtoms();
    }
    
    void XWaylandIntegration::initializeAtoms()
    {
        // Definiere die Atome, die wir brauchen
        static const char* atomNames[] = {
            "_NET_WM_NAME",
            "_NET_WM_VISIBLE_NAME",
            "_NET_WM_ICON_NAME",
            "_NET_WM_VISIBLE_ICON_NAME",
            "_NET_WM_WINDOW_TYPE",
            "_NET_WM_WINDOW_TYPE_NORMAL",
            "_NET_WM_WINDOW_TYPE_DIALOG",
            "_NET_WM_WINDOW_TYPE_MENU",
            "_NET_WM_WINDOW_TYPE_TOOLBAR",
            "_NET_WM_WINDOW_TYPE_SPLASH",
            "_NET_WM_WINDOW_TYPE_UTILITY",
            "_NET_WM_WINDOW_TYPE_DROPDOWN_MENU",
            "_NET_WM_WINDOW_TYPE_POPUP_MENU",
            "_NET_WM_WINDOW_TYPE_TOOLTIP",
            "_NET_WM_WINDOW_TYPE_NOTIFICATION",
            "_NET_WM_STATE",
            "_NET_WM_STATE_MODAL",
            "_NET_WM_STATE_STICKY",
            "_NET_WM_STATE_MAXIMIZED_VERT",
            "_NET_WM_STATE_MAXIMIZED_HORZ",
            "_NET_WM_STATE_FULLSCREEN",
            "_NET_SUPPORTED",
            "_NET_CLIENT_LIST",
            "_NET_WM_PID",
            "_NET_ACTIVE_WINDOW",
            "WM_PROTOCOLS",
            "WM_DELETE_WINDOW",
            "_NET_WM_PING",
            "_NET_WM_SYNC_REQUEST",
            "_NET_WM_SYNC_REQUEST_COUNTER"
        };
        
        // Erstelle die Anfrage für alle Atome
        xcb_intern_atom_cookie_t cookies[sizeof(atomNames) / sizeof(atomNames[0])];
        for (size_t i = 0; i < sizeof(atomNames) / sizeof(atomNames[0]); i++) {
            cookies[i] = xcb_intern_atom(m_xcbConnection, 0, strlen(atomNames[i]), atomNames[i]);
        }
        
        // Verarbeite die Antworten und speichere die Atome
        for (size_t i = 0; i < sizeof(atomNames) / sizeof(atomNames[0]); i++) {
            xcb_intern_atom_reply_t* reply = xcb_intern_atom_reply(m_xcbConnection, cookies[i], nullptr);
            if (reply) {
                m_atoms.insert(QString(atomNames[i]), reply->atom);
                free(reply);
            }
        }
        
        // Setze die unterstützten Fenstermanagement-Features
        xcb_screen_t* screen = xcb_setup_roots_iterator(xcb_get_setup(m_xcbConnection)).data;
        if (screen) {
            QVector<xcb_atom_t> supported;
            
            // Füge die unterstützten Atome hinzu
            supported << m_atoms.value("_NET_WM_NAME")
                     << m_atoms.value("_NET_WM_VISIBLE_NAME")
                     << m_atoms.value("_NET_WM_WINDOW_TYPE")
                     << m_atoms.value("_NET_WM_STATE")
                     << m_atoms.value("_NET_WM_STATE_FULLSCREEN")
                     << m_atoms.value("_NET_WM_STATE_MAXIMIZED_VERT")
                     << m_atoms.value("_NET_WM_STATE_MAXIMIZED_HORZ")
                     << m_atoms.value("_NET_ACTIVE_WINDOW")
                     << m_atoms.value("_NET_WM_PID");
            
            // Setze die unterstützten Atome auf dem Root-Fenster
            xcb_change_property(
                m_xcbConnection,
                XCB_PROP_MODE_REPLACE,
                screen->root,
                m_atoms.value("_NET_SUPPORTED"),
                XCB_ATOM_ATOM,
                32,
                supported.size(),
                supported.constData()
            );
            
            xcb_flush(m_xcbConnection);
        }
    }
    
    void XWaylandIntegration::cleanupAfterFailure()
    {
        // Bereinige alle Ressourcen nach einem Fehler beim Starten
        
        if (m_xwaylandProcess) {
            if (m_xwaylandProcess->state() == QProcess::Running) {
                m_xwaylandProcess->terminate();
                if (!m_xwaylandProcess->waitForFinished(5000)) {
                    m_xwaylandProcess->kill();
                    m_xwaylandProcess->waitForFinished(1000);
                }
            }
            
            delete m_xwaylandProcess;
            m_xwaylandProcess = nullptr;
        }
        
        // Schließe Socket-Paare
        if (m_wmFd >= 0) {
            close(m_wmFd);
            m_wmFd = -1;
        }
        
        if (m_displayFd >= 0) {
            close(m_displayFd);
            m_displayFd = -1;
        }
        
        // Lösche XCB-Verbindung
        if (m_xcbConnection) {
            xcb_disconnect(m_xcbConnection);
            m_xcbConnection = nullptr;
        }
        
        // Deaktiviere Laufzeitstatus
        m_running = false;
    }
    
    QString XWaylandIntegration::generateXauthFile() const
    {
        // Erstelle einen temporären Pfad für die Xauthority-Datei
        QString xauthPath = QDir::tempPath() + "/xwayland-xauth-" + 
                          QString::number(QCoreApplication::applicationPid());
        
        // Generiere einen zufälligen Cookie für die Authentifizierung
        QByteArray cookie;
        cookie.resize(16);
        for (int i = 0; i < 16; i++) {
            cookie[i] = static_cast<char>(QRandomGenerator::global()->bounded(256));
        }
        
        // Erstelle die xauth-Datei
        QFile file(xauthPath);
        if (file.open(QIODevice::WriteOnly)) {
            // Hier müsste der korrekte xauth-Protokollcode stehen
            // Dies ist eine vereinfachte Version
            file.write(QString("0100000001000000000000000000" + QString::number(m_displayNumber) + "00000000").toUtf8());
            file.write(cookie);
            file.close();
            
            // Setze die Berechtigungen
            QFile::setPermissions(xauthPath, QFile::ReadOwner | QFile::WriteOwner);
        } else {
            Core::Logger::instance().error(
                QString("Konnte Xauth-Datei nicht erstellen: %1").arg(file.errorString()),
                "Wayland"
            );
        }
        
        return xauthPath;
    }
    
    } // namespace Wayland
    } // namespace VivoX
    