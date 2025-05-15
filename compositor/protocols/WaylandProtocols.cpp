/**
 * @file WaylandProtocols.cpp
 * @brief Implementierung der Wayland-Protokoll-Manager-Klasse
 * 
 * Diese Datei enthält die Implementierung der ProtocolManager-Klasse, die für die
 * Verwaltung und Registrierung aller Wayland-Protokolle verantwortlich ist.
 */

 #include "WaylandProtocols.h"
 #include "../core/Logger.h"
 #include <QWaylandSurface>
 #include <QWaylandCompositor>
 #include <QWaylandXdgSurface>
 #include <QWaylandXdgToplevel>
 #include <QWaylandXdgPopup>
 #include <QWaylandIviSurface>
 #include <QWaylandQuickShellSurfaceItem>
 #include <QWaylandShellSurfaceItem>
 #include <QProcess>
 #include <QStandardPaths>
 #include <stdexcept>
 
 namespace VivoX {
 namespace Wayland {
 
 // Private implementation class for XWayland support
 class XWaylandManager : public QObject {
     Q_OBJECT
 public:
     explicit XWaylandManager(QObject* parent = nullptr);
     ~XWaylandManager();
 
     bool initialize();
     bool isRunning() const;
     void shutdown();
 
 signals:
     void started();
     void stopped();
     void error(const QString& errorMessage);
 
 private:
     QProcess* m_xwaylandProcess;
     bool m_isRunning;
 };
 
 // Private implementation for Layer Shell protocol
 class LayerShellManager : public QObject {
     Q_OBJECT
 public:
     explicit LayerShellManager(QWaylandCompositor* compositor, QObject* parent = nullptr);
     ~LayerShellManager();
 
     bool initialize();
 
     // Layer shell specific methods
     enum Layer {
         BackgroundLayer = 0,
         BottomLayer = 1,
         TopLayer = 2,
         OverlayLayer = 3
     };
 
     void setLayerForSurface(QWaylandSurface* surface, Layer layer);
     Layer getLayerForSurface(QWaylandSurface* surface) const;
 
 signals:
     void layerSurfaceCreated(QWaylandSurface* surface, Layer layer);
     void layerSurfaceDestroyed(QWaylandSurface* surface);
 
 private:
     QWaylandCompositor* m_compositor;
     QMap<QWaylandSurface*, Layer> m_surfaceLayers;
 };
 
 // Private implementation for Presentation Time protocol
 class PresentationTimeManager : public QObject {
     Q_OBJECT
 public:
     explicit PresentationTimeManager(QWaylandCompositor* compositor, QObject* parent = nullptr);
     ~PresentationTimeManager();
 
     bool initialize();
     void framePresentedEvent(QWaylandSurface* surface, quint64 presentationTime);
 
 signals:
     void presentationTimeSupported(QWaylandSurface* surface);
 
 private:
     QWaylandCompositor* m_compositor;
     QMap<QWaylandSurface*, bool> m_surfaceSupport;
 };
 
 // Private implementation for Viewporter protocol
 class ViewporterManager : public QObject {
     Q_OBJECT
 public:
     explicit ViewporterManager(QWaylandCompositor* compositor, QObject* parent = nullptr);
     ~ViewporterManager();
 
     bool initialize();
     void setViewportDestination(QWaylandSurface* surface, const QSize& destinationSize);
     QSize getViewportDestination(QWaylandSurface* surface) const;
 
 signals:
     void viewportChanged(QWaylandSurface* surface, const QSize& size);
 
 private:
     QWaylandCompositor* m_compositor;
     QMap<QWaylandSurface*, QSize> m_surfaceViewports;
 };
 
 ProtocolManager::ProtocolManager(QObject* parent)
     : QObject(parent)
     , m_compositor(nullptr)
     , m_xdgShell(nullptr)
     , m_xdgDecorationManager(nullptr)
     , m_qtWindowManager(nullptr)
     , m_iviApplication(nullptr)
     , m_textInputManager(nullptr)
     , m_linuxDmabuf(nullptr)
     , m_layerShell(nullptr)
     , m_presentationTime(nullptr)
     , m_viewporter(nullptr)
     , m_xwayland(nullptr)
 {
     Core::Logger::instance().info("ProtocolManager erstellt", "Wayland");
 }
 
 ProtocolManager::~ProtocolManager()
 {
     // Clean up XWayland if needed
     if (m_xwayland) {
         static_cast<XWaylandManager*>(m_xwayland)->shutdown();
     }
     
     Core::Logger::instance().info("ProtocolManager zerstört", "Wayland");
 }
 
 ProtocolManager& ProtocolManager::instance()
 {
     static ProtocolManager instance;
     return instance;
 }
 
 bool ProtocolManager::initialize(QWaylandCompositor* compositor)
 {
     if (!compositor) {
         Core::Logger::instance().error("Ungültiger Compositor", "Wayland");
         return false;
     }
     
     m_compositor = compositor;
     
     Core::Logger::instance().info("ProtocolManager initialisiert", "Wayland");
     return true;
 }
 
 bool ProtocolManager::registerStandardProtocols()
 {
     Core::Logger::instance().info("Registriere Standard-Protokolle", "Wayland");
     
     bool success = true;
     
     // Registriere XDG-Shell
     success &= registerXdgShell();
     
     // Registriere XDG-Decoration
     success &= registerXdgDecoration();
     
     // Registriere Qt-Window-Manager
     success &= registerQtWindowManager();
     
     // Registriere Text-Input
     success &= registerTextInput();
     
     Core::Logger::instance().info("Standard-Protokolle registriert", "Wayland");
     return success;
 }
 
 bool ProtocolManager::registerExtensionProtocols()
 {
     Core::Logger::instance().info("Registriere Erweiterungs-Protokolle", "Wayland");
     
     bool success = true;
     
     // Registriere IVI-Application
     success &= registerIviApplication();
     
     // Registriere Linux-DMABUF
     success &= registerLinuxDmabuf();
     
     // Registriere Layer-Shell
     success &= registerLayerShell();
     
     // Registriere Presentation-Time
     success &= registerPresentationTime();
     
     // Registriere Viewporter
     success &= registerViewporter();
     
     // Registriere XWayland
     success &= registerXWayland();
     
     Core::Logger::instance().info("Erweiterungs-Protokolle registriert", "Wayland");
     return success;
 }
 
 QWaylandXdgShell* ProtocolManager::xdgShell() const
 {
     return m_xdgShell;
 }
 
 QWaylandXdgDecorationManagerV1* ProtocolManager::xdgDecorationManager() const
 {
     return m_xdgDecorationManager;
 }
 
 QWaylandQtWindowManager* ProtocolManager::qtWindowManager() const
 {
     return m_qtWindowManager;
 }
 
 QWaylandIviApplication* ProtocolManager::iviApplication() const
 {
     return m_iviApplication;
 }
 
 QWaylandTextInputManager* ProtocolManager::textInputManager() const
 {
     return m_textInputManager;
 }
 
 QWaylandLinuxDmabufClientBufferIntegration* ProtocolManager::linuxDmabuf() const
 {
     return m_linuxDmabuf;
 }
 
 LayerShellManager* ProtocolManager::layerShell() const
 {
     return static_cast<LayerShellManager*>(m_layerShell);
 }
 
 PresentationTimeManager* ProtocolManager::presentationTime() const
 {
     return static_cast<PresentationTimeManager*>(m_presentationTime);
 }
 
 ViewporterManager* ProtocolManager::viewporter() const
 {
     return static_cast<ViewporterManager*>(m_viewporter);
 }
 
 XWaylandManager* ProtocolManager::xwayland() const
 {
     return static_cast<XWaylandManager*>(m_xwayland);
 }
 
 bool ProtocolManager::registerXdgShell()
 {
     if (!m_compositor) {
         Core::Logger::instance().error("Compositor nicht initialisiert", "Wayland");
         return false;
     }
     
     try {
         m_xdgShell = new QWaylandXdgShell(m_compositor);
         
         // Verbinde Signale
         connect(m_xdgShell, &QWaylandXdgShell::xdgSurfaceCreated, this, [this](QWaylandXdgSurface* xdgSurface) {
             Core::Logger::instance().debug(QString("XDG-Surface erstellt: %1").arg(xdgSurface->surface()->client()->processId()), "Wayland");
             
             // Verbinde Signale für XDG-Surface
             connect(xdgSurface, &QWaylandXdgSurface::toplevelCreated, this, [this](QWaylandXdgToplevel* toplevel) {
                 Core::Logger::instance().debug(QString("XDG-Toplevel erstellt: %1").arg(toplevel->xdgSurface()->surface()->client()->processId()), "Wayland");
                 
                 // Verbinde Signale für XDG-Toplevel
                 connect(toplevel, &QWaylandXdgToplevel::titleChanged, this, [toplevel]() {
                     Core::Logger::instance().debug(QString("XDG-Toplevel Titel geändert: %1").arg(toplevel->title()), "Wayland");
                 });
                 
                 connect(toplevel, &QWaylandXdgToplevel::appIdChanged, this, [toplevel]() {
                     Core::Logger::instance().debug(QString("XDG-Toplevel App-ID geändert: %1").arg(toplevel->appId()), "Wayland");
                 });
                 
                 connect(toplevel, &QWaylandXdgToplevel::maximizedChanged, this, [toplevel]() {
                     Core::Logger::instance().debug(QString("XDG-Toplevel Maximierung geändert: %1").arg(toplevel->maximized() ? "maximiert" : "nicht maximiert"), "Wayland");
                 });
                 
                 connect(toplevel, &QWaylandXdgToplevel::fullscreenChanged, this, [toplevel]() {
                     Core::Logger::instance().debug(QString("XDG-Toplevel Vollbild geändert: %1").arg(toplevel->fullscreen() ? "Vollbild" : "kein Vollbild"), "Wayland");
                 });
                 
                 connect(toplevel, &QWaylandXdgToplevel::minimizedChanged, this, [toplevel]() {
                     Core::Logger::instance().debug(QString("XDG-Toplevel Minimierung geändert: %1").arg(toplevel->minimized() ? "minimiert" : "nicht minimiert"), "Wayland");
                 });
                 
                 connect(toplevel, &QWaylandXdgToplevel::startMove, this, [toplevel]() {
                     Core::Logger::instance().debug(QString("XDG-Toplevel Bewegung gestartet: %1").arg(toplevel->title()), "Wayland");
                 });
                 
                 connect(toplevel, &QWaylandXdgToplevel::startResize, this, [toplevel](QWaylandSeat* seat, Qt::Edges edges) {
                     Q_UNUSED(seat);
                     QString edgesStr;
                     if (edges & Qt::TopEdge) edgesStr += "oben ";
                     if (edges & Qt::BottomEdge) edgesStr += "unten ";
                     if (edges & Qt::LeftEdge) edgesStr += "links ";
                     if (edges & Qt::RightEdge) edgesStr += "rechts ";
                     Core::Logger::instance().debug(QString("XDG-Toplevel Größenänderung gestartet: %1, Kanten: %2").arg(toplevel->title()).arg(edgesStr), "Wayland");
                 });
                 
                 emit topLevelSurfaceCreated(toplevel);
             });
             
             connect(xdgSurface, &QWaylandXdgSurface::popupCreated, this, [this](QWaylandXdgPopup* popup) {
                 Core::Logger::instance().debug(QString("XDG-Popup erstellt: %1").arg(popup->xdgSurface()->surface()->client()->processId()), "Wayland");
                 
                 // Verbinde Signale für XDG-Popup
                 connect(popup, &QWaylandXdgPopup::grabRequested, this, [popup](QWaylandSeat* seat, uint serial) {
                     Q_UNUSED(seat);
                     Core::Logger::instance().debug(QString("XDG-Popup Grab angefordert: Serial %1").arg(serial), "Wayland");
                     popup->sendConfigure(popup->configuredGeometry());
                 });
                 
                 connect(popup, &QWaylandXdgPopup::configuredGeometryChanged, this, [popup]() {
                     Core::Logger::instance().debug(QString("XDG-Popup Geometrie geändert: %1,%2 %3x%4")
                         .arg(popup->configuredGeometry().x())
                         .arg(popup->configuredGeometry().y())
                         .arg(popup->configuredGeometry().width())
                         .arg(popup->configuredGeometry().height()), "Wayland");
                 });
                 
                 emit popupSurfaceCreated(popup);
             });
             
             emit xdgSurfaceCreated(xdgSurface);
         });
         
         emit protocolRegistered("xdg-shell");
         Core::Logger::instance().info("XDG-Shell registriert", "Wayland");
         return true;
     } catch (const std::exception& e) {
         Core::Logger::instance().error(QString("Fehler beim Registrieren von XDG-Shell: %1").arg(e.what()), "Wayland");
         return false;
     }
 }
 
 bool ProtocolManager::registerXdgDecoration()
 {
     if (!m_compositor || !m_xdgShell) {
         Core::Logger::instance().error("Compositor oder XDG-Shell nicht initialisiert", "Wayland");
         return false;
     }
     
     try {
         m_xdgDecorationManager = new QWaylandXdgDecorationManagerV1(m_compositor);
         
         // Verbinde Signale
         connect(m_xdgDecorationManager, &QWaylandXdgDecorationManagerV1::decorationCreated, this, 
             [this](QWaylandXdgToplevelDecorationV1 *decoration) {
                 Core::Logger::instance().debug("XDG-Decoration erstellt", "Wayland");
                 
                 // Setze den Standard-Modus auf Server-Side-Decoration
                 decoration->setMode(QWaylandXdgToplevelDecorationV1::Mode::ServerSide);
                 
                 connect(decoration, &QWaylandXdgToplevelDecorationV1::preferredModeChanged, this, 
                     [decoration]() {
                         QString mode;
                         switch (decoration->preferredMode()) {
                             case QWaylandXdgToplevelDecorationV1::Mode::ClientSide:
                                 mode = "Client-Side";
                                 break;
                             case QWaylandXdgToplevelDecorationV1::Mode::ServerSide:
                                 mode = "Server-Side";
                                 break;
                             default:
                                 mode = "Unbekannt";
                         }
                         Core::Logger::instance().debug(QString("XDG-Decoration bevorzugter Modus geändert: %1").arg(mode), "Wayland");
                         
                         // Server entscheidet über den Modus basierend auf der Präferenz des Clients
                         // Wir respektieren den Wunsch des Clients
                         decoration->setMode(decoration->preferredMode());
                     });
                 
                 emit decorationCreated(decoration);
             });
         
         emit protocolRegistered("xdg-decoration");
         Core::Logger::instance().info("XDG-Decoration registriert", "Wayland");
         return true;
     } catch (const std::exception& e) {
         Core::Logger::instance().error(QString("Fehler beim Registrieren von XDG-Decoration: %1").arg(e.what()), "Wayland");
         return false;
     }
 }
 
 bool ProtocolManager::registerQtWindowManager()
 {
     if (!m_compositor) {
         Core::Logger::instance().error("Compositor nicht initialisiert", "Wayland");
         return false;
     }
     
     try {
         m_qtWindowManager = new QWaylandQtWindowManager(m_compositor);
         
         // Verbinde Signale
         connect(m_qtWindowManager, &QWaylandQtWindowManager::showIsFullScreenChanged, this, 
             [this]() {
                 Core::Logger::instance().debug(QString("Qt-Window-Manager Vollbildanzeige geändert: %1")
                     .arg(m_qtWindowManager->showIsFullScreen() ? "Vollbild" : "kein Vollbild"), "Wayland");
             });
         
         // Qt-spezifische Funktionen
         m_qtWindowManager->setShowIsFullScreen(false);
         
         emit protocolRegistered("qt-window-manager");
         Core::Logger::instance().info("Qt-Window-Manager registriert", "Wayland");
         return true;
     } catch (const std::exception& e) {
         Core::Logger::instance().error(QString("Fehler beim Registrieren von Qt-Window-Manager: %1").arg(e.what()), "Wayland");
         return false;
     }
 }
 
 bool ProtocolManager::registerIviApplication()
 {
     if (!m_compositor) {
         Core::Logger::instance().error("Compositor nicht initialisiert", "Wayland");
         return false;
     }
     
     try {
         m_iviApplication = new QWaylandIviApplication(m_compositor);
         
         // Verbinde Signale
         connect(m_iviApplication, &QWaylandIviApplication::iviSurfaceCreated, this, 
             [this](QWaylandIviSurface* iviSurface) {
                 Core::Logger::instance().debug(QString("IVI-Surface erstellt: ID %1")
                     .arg(iviSurface->iviId()), "Wayland");
                 
                 // Verbinde Signale für IVI-Surface
                 connect(iviSurface->surface(), &QWaylandSurface::hasContentChanged, this, 
                     [iviSurface]() {
                         Core::Logger::instance().debug(QString("IVI-Surface %1 Inhalt geändert: %2")
                             .arg(iviSurface->iviId())
                             .arg(iviSurface->surface()->hasContent() ? "mit Inhalt" : "ohne Inhalt"), "Wayland");
                     });
                 
                 // Wir könnten hier weitere IVI-spezifische Verarbeitung hinzufügen
                 
                 emit iviSurfaceCreated(iviSurface);
             });
         
         emit protocolRegistered("ivi-application");
         Core::Logger::instance().info("IVI-Application registriert", "Wayland");
         return true;
     } catch (const std::exception& e) {
         Core::Logger::instance().error(QString("Fehler beim Registrieren von IVI-Application: %1").arg(e.what()), "Wayland");
         return false;
     }
 }
 
 bool ProtocolManager::registerTextInput()
 {
     if (!m_compositor) {
         Core::Logger::instance().error("Compositor nicht initialisiert", "Wayland");
         return false;
     }
     
     try {
         m_textInputManager = new QWaylandTextInputManager(m_compositor);
         
         // Verbinde Signale für Text-Input
         connect(m_compositor, &QWaylandCompositor::defaultSeatChanged, this, 
             [this](QWaylandSeat *seat) {
                 if (seat) {
                     connect(seat, &QWaylandSeat::keyboardFocusChanged, this, 
                         [this, seat](QWaylandSurface *surface) {
                             if (surface) {
                                 Core::Logger::instance().debug(QString("Tastatur-Fokus geändert: PID %1")
                                     .arg(surface->client()->processId()), "Wayland");
                                 
                                 // Hier können wir Text-Input-spezifische Verarbeitung hinzufügen
                                 // z.B. virtuellen Keyboard anzeigen oder Text-Input-Protokoll aktivieren
                             }
                         });
                 }
             });
         
         emit protocolRegistered("text-input");
         Core::Logger::instance().info("Text-Input registriert", "Wayland");
         return true;
     } catch (const std::exception& e) {
         Core::Logger::instance().error(QString("Fehler beim Registrieren von Text-Input: %1").arg(e.what()), "Wayland");
         return false;
     }
 }
 
 bool ProtocolManager::registerLinuxDmabuf()
 {
     if (!m_compositor) {
         Core::Logger::instance().error("Compositor nicht initialisiert", "Wayland");
         return false;
     }
     
     try {
         m_linuxDmabuf = new QWaylandLinuxDmabufClientBufferIntegration(m_compositor->display());
         
         // Format-Unterstützung registrieren
         const QVector<uint32_t> formats = {
             DRM_FORMAT_ARGB8888,
             DRM_FORMAT_XRGB8888,
             DRM_FORMAT_RGBA8888,
             DRM_FORMAT_RGBX8888,
             DRM_FORMAT_RGB888,
             DRM_FORMAT_RGB565
         };
         
         // Registriere DMA-BUF-Formate mit verschiedenen Modifiern
         for (uint32_t format : formats) {
             // Unterstützt lineare Speicherung (kein Modifier)
             m_linuxDmabuf->addSupportedFormat(format, DRM_FORMAT_MOD_LINEAR);
             
             // Unterstützt weitere Modifikatoren je nach Hardware-Unterstützung
             // Wir fügen hier gängige Modifikatoren hinzu, aber in einer echten Implementierung
             // würden diese von der tatsächlichen GPU-Hardware abhängen
             m_linuxDmabuf->addSupportedFormat(format, DRM_FORMAT_MOD_INVALID);
         }
         
         emit protocolRegistered("linux-dmabuf");
         Core::Logger::instance().info("Linux-DMABUF registriert", "Wayland");
         return true;
     } catch (const std::exception& e) {
         Core::Logger::instance().error(QString("Fehler beim Registrieren von Linux-DMABUF: %1").arg(e.what()), "Wayland");
         return false;
     }
 }
 
 bool ProtocolManager::registerLayerShell()
 {
     if (!m_compositor) {
         Core::Logger::instance().error("Compositor nicht initialisiert", "Wayland");
         return false;
     }
     
     try {
         // Implementiere Layer-Shell-Protokoll
         m_layerShell = new LayerShellManager(m_compositor, this);
         
         if (!static_cast<LayerShellManager*>(m_layerShell)->initialize()) {
             Core::Logger::instance().error("Fehler beim Initialisieren des Layer-Shell-Managers", "Wayland");
             delete m_layerShell;
             m_layerShell = nullptr;
             return false;
         }
         
         // Verbinde Signale
         connect(static_cast<LayerShellManager*>(m_layerShell), &LayerShellManager::layerSurfaceCreated, this,
             [this](QWaylandSurface* surface, LayerShellManager::Layer layer) {
                 QString layerName;
                 switch (layer) {
                     case LayerShellManager::BackgroundLayer:
                         layerName = "Hintergrund";
                         break;
                     case LayerShellManager::BottomLayer:
                         layerName = "Unten";
                         break;
                     case LayerShellManager::TopLayer:
                         layerName = "Oben";
                         break;
                     case LayerShellManager::OverlayLayer:
                         layerName = "Overlay";
                         break;
                     default:
                         layerName = "Unbekannt";
                 }
                 Core::Logger::instance().debug(QString("Layer-Shell Surface erstellt: PID %1, Layer %2")
                     .arg(surface->client()->processId())
                     .arg(layerName), "Wayland");
             });
         
         emit protocolRegistered("layer-shell");
         Core::Logger::instance().info("Layer-Shell registriert", "Wayland");
         return true;
     } catch (const std::exception& e) {
         Core::Logger::instance().error(QString("Fehler beim Registrieren von Layer-Shell: %1").arg(e.what()), "Wayland");
         return false;
     }
 }
 
 bool ProtocolManager::registerPresentationTime()
 {
     if (!m_compositor) {
         Core::Logger::instance().error("Compositor nicht initialisiert", "Wayland");
         return false;
     }
     
     try {
         // Implementiere Presentation-Time-Protokoll
         m_presentationTime = new PresentationTimeManager(m_compositor, this);
         
         if (!static_cast<PresentationTimeManager*>(m_presentationTime)->initialize()) {
             Core::Logger::instance().error("Fehler beim Initialisieren des Presentation-Time-Managers", "Wayland");
             delete m_presentationTime;
             m_presentationTime = nullptr;
             return false;
         }
         
         // Verbinde Signale
         connect(static_cast<PresentationTimeManager*>(m_presentationTime), &PresentationTimeManager::presentationTimeSupported, this,
             [](QWaylandSurface* surface) {
                 Core::Logger::instance().debug(QString("Presentation-Time unterstützt für Surface: PID %1")
                     .arg(surface->client()->processId()), "Wayland");
             });
         
         emit protocolRegistered("presentation-time");
         Core::Logger::instance().info("Presentation-Time registriert", "Wayland");
         return true;
     } catch (const std::exception& e) {
         Core::Logger::instance().error(QString("Fehler beim Registrieren von Presentation-Time: %1").arg(e.what()), "Wayland");
         return false;
     }
 }
 
 bool ProtocolManager::registerViewporter()
 {
     if (!m_compositor) {
         Core::Logger::instance().error("Compositor nicht initialisiert", "Wayland");
         return false;
     }
     
     try {
         // Implementiere Viewporter-Protokoll
         m_viewporter = new ViewporterManager(m_compositor, this);
         
         if (!static_cast<ViewporterManager*>(m_viewporter)->initialize()) {
             Core::Logger::instance().error("Fehler beim Initialisieren des Viewporter-Managers", "Wayland");
             delete m_viewporter;
             m_viewporter = nullptr;
             return false;
         }
         
         // Verbinde Signale
         connect(static_cast<ViewporterManager*>(m_viewporter), &ViewporterManager::viewportChanged, this,
             [](QWaylandSurface* surface, const QSize& size) {
                 Core::Logger::instance().debug(QString("Viewport geändert für Surface: PID %1, Größe %2x%3")
                     .arg(surface->client()->processId())
                     .arg(size.width())
                     .arg(size.height()), "Wayland");
             });
         
         emit protocolRegistered("viewporter");
         Core::Logger::instance().info("Viewporter registriert", "Wayland");
         return true;
     } catch (const std::exception& e) {
         Core::Logger::instance().error(QString("Fehler beim Registrieren von Viewporter: %1").arg(e.what()), "Wayland");
         return false;
     }
 }
 
 bool ProtocolManager::registerXWayland()
 {
     if (!m_compositor) {
         Core::Logger::instance().error("Compositor nicht initialisiert", "Wayland");
         return false;
     }
     
     try {
         // Implementiere XWayland-Integration
         m_xwayland = new XWaylandManager(this);
         
         if (!static_cast<XWaylandManager*>(m_xwayland)->initialize()) {
             Core::Logger::instance().error("Fehler beim Initialisieren von XWayland", "Wayland");
             delete m_xwayland;
             m_xwayland = nullptr;
             return false;
         }
         
         // Verbinde Signale
         connect(static_cast<XWaylandManager*>(m_xwayland), &XWaylandManager::started, this,
             []() {
                 Core::Logger::instance().info("XWayland gestartet", "Wayland");
             });
         
         connect(static_cast<XWaylandManager*>(m_xwayland), &XWaylandManager::stopped, this,
             []() {
                 Core::Logger::instance().info("XWayland gestoppt", "Wayland");
             });
         
         connect(static_cast<XWaylandManager*>(m_xwayland), &XWaylandManager::error, this,
             [](const QString& errorMessage) {
                 Core::Logger::instance().error(QString("XWayland-Fehler: %1").arg(errorMessage), "Wayland");
             });
         
         emit protocolRegistered("xwayland");
         Core::Logger::instance().info("XWayland registriert", "Wayland");
         return true;
     } catch (const std::exception& e) {
         Core::Logger::instance().error(QString("Fehler beim Registrieren von XWayland: %1").arg(e.what()), "Wayland");
         return false;
     }
 }
 
// Implementation of XWaylandManager class
XWaylandManager::XWaylandManager(QObject* parent)
    : QObject(parent)
    , m_xwaylandProcess(nullptr)
    , m_isRunning(false)
{
}

XWaylandManager::~XWaylandManager()
{
    shutdown();
}

bool XWaylandManager::initialize()
{
    // Prüfe, ob XWayland bereits läuft
    if (m_isRunning) {
        return true;
    }
    
    // Prüfe, ob XWayland verfügbar ist
    QString xwaylandPath = QStandardPaths::findExecutable("Xwayland");
    if (xwaylandPath.isEmpty()) {
        emit error("XWayland wurde im System nicht gefunden");
        return false;
    }
    
    // Erstelle Prozess für XWayland
    m_xwaylandProcess = new QProcess(this);
    
    // Setze Umgebungsvariablen für XWayland
    QProcessEnvironment env = QProcessEnvironment::systemEnvironment();
    env.insert("WAYLAND_DISPLAY", m_compositor->socketName());
    m_xwaylandProcess->setProcessEnvironment(env);
    
    // Verbinde Signale für Prozessbehandlung
    connect(m_xwaylandProcess, &QProcess::started, this, [this]() {
        m_isRunning = true;
        emit started();
    });
    
    connect(m_xwaylandProcess, QOverload<int, QProcess::ExitStatus>::of(&QProcess::finished), this, 
        [this](int exitCode, QProcess::ExitStatus exitStatus) {
            Q_UNUSED(exitStatus);
            m_isRunning = false;
            emit stopped();
            
            if (exitCode != 0) {
                emit error(QString("XWayland beendet mit Fehlercode: %1").arg(exitCode));
            }
        });
    
    connect(m_xwaylandProcess, &QProcess::errorOccurred, this, [this](QProcess::ProcessError error) {
        QString errorMessage;
        switch (error) {
            case QProcess::FailedToStart:
                errorMessage = "XWayland konnte nicht gestartet werden";
                break;
            case QProcess::Crashed:
                errorMessage = "XWayland ist abgestürzt";
                break;
            case QProcess::Timedout:
                errorMessage = "Zeitüberschreitung beim Starten von XWayland";
                break;
            case QProcess::WriteError:
                errorMessage = "Fehler beim Schreiben an den XWayland-Prozess";
                break;
            case QProcess::ReadError:
                errorMessage = "Fehler beim Lesen vom XWayland-Prozess";
                break;
            default:
                errorMessage = "Unbekannter Fehler beim XWayland-Prozess";
        }
        
        m_isRunning = false;
        emit this->error(errorMessage);
    });
    
    // Starte XWayland mit angemessenen Parametern
    QStringList args;
    args << "-rootless" << "-terminate" << "-displayfd" << "3";
    m_xwaylandProcess->start(xwaylandPath, args);
    
    // Warte bis zu 5 Sekunden auf den Start
    if (!m_xwaylandProcess->waitForStarted(5000)) {
        emit error("Zeitüberschreitung beim Starten von XWayland");
        delete m_xwaylandProcess;
        m_xwaylandProcess = nullptr;
        return false;
    }
    
    return true;
}

bool XWaylandManager::isRunning() const
{
    return m_isRunning;
}

void XWaylandManager::shutdown()
{
    if (m_xwaylandProcess) {
        if (m_isRunning) {
            // Versuche, den Prozess ordnungsgemäß zu beenden
            m_xwaylandProcess->terminate();
            
            // Warte bis zu 3 Sekunden auf die Beendigung
            if (!m_xwaylandProcess->waitForFinished(3000)) {
                // Wenn es nicht reagiert, erzwinge die Beendigung
                m_xwaylandProcess->kill();
            }
        }
        
        delete m_xwaylandProcess;
        m_xwaylandProcess = nullptr;
        m_isRunning = false;
    }
}

// Implementation of LayerShellManager class
LayerShellManager::LayerShellManager(QWaylandCompositor* compositor, QObject* parent)
    : QObject(parent)
    , m_compositor(compositor)
{
}

LayerShellManager::~LayerShellManager()
{
}

bool LayerShellManager::initialize()
{
    if (!m_compositor) {
        return false;
    }
    
    // Verbinde das Signal für neue Oberflächen
    connect(m_compositor, &QWaylandCompositor::surfaceCreated, this, [this](QWaylandSurface* surface) {
        // Standardmäßig ist die Oberfläche in keiner Ebene
        m_surfaceLayers.insert(surface, TopLayer); // Standard ist TopLayer
        
        // Verbinde das Signal zum Entfernen der Oberfläche
        connect(surface, &QWaylandSurface::surfaceDestroyed, this, [this, surface]() {
            if (m_surfaceLayers.contains(surface)) {
                Layer layer = m_surfaceLayers.take(surface);
                emit layerSurfaceDestroyed(surface);
            }
        });
    });
    
    return true;
}

void LayerShellManager::setLayerForSurface(QWaylandSurface* surface, Layer layer)
{
    if (!surface) {
        return;
    }
    
    // Speichere die Ebene für die Oberfläche
    m_surfaceLayers[surface] = layer;
    
    // Signalisiere die Änderung
    emit layerSurfaceCreated(surface, layer);
}

LayerShellManager::Layer LayerShellManager::getLayerForSurface(QWaylandSurface* surface) const
{
    if (!surface || !m_surfaceLayers.contains(surface)) {
        return TopLayer; // Standardwert, wenn keine Ebene festgelegt ist
    }
    
    return m_surfaceLayers.value(surface);
}

// Implementation of PresentationTimeManager class
PresentationTimeManager::PresentationTimeManager(QWaylandCompositor* compositor, QObject* parent)
    : QObject(parent)
    , m_compositor(compositor)
{
}

PresentationTimeManager::~PresentationTimeManager()
{
}

bool PresentationTimeManager::initialize()
{
    if (!m_compositor) {
        return false;
    }
    
    // Verbinde das Signal für neue Oberflächen
    connect(m_compositor, &QWaylandCompositor::surfaceCreated, this, [this](QWaylandSurface* surface) {
        // Markiere die Oberfläche als präsentationszeitfähig
        m_surfaceSupport.insert(surface, true);
        
        // Signalisiere die Unterstützung
        emit presentationTimeSupported(surface);
        
        // Verbinde das Signal zum Entfernen der Oberfläche
        connect(surface, &QWaylandSurface::surfaceDestroyed, this, [this, surface]() {
            m_surfaceSupport.remove(surface);
        });
    });
    
    return true;
}

void PresentationTimeManager::framePresentedEvent(QWaylandSurface* surface, quint64 presentationTime)
{
    if (!surface || !m_surfaceSupport.contains(surface) || !m_surfaceSupport.value(surface)) {
        // Die Oberfläche unterstützt keine Präsentationszeit
        return;
    }
    
    // Hier würden wir den präsentierten Frame mit der Präsentationszeit an die Anwendung melden
    // Dies erfordert eine tiefere Integration mit dem Compositor-Code, daher ist dies nur ein Platzhalter
    Core::Logger::instance().debug(QString("Frame präsentiert für Surface: PID %1, Zeit %2")
        .arg(surface->client()->processId())
        .arg(presentationTime), "Wayland");
}

// Implementation of ViewporterManager class
ViewporterManager::ViewporterManager(QWaylandCompositor* compositor, QObject* parent)
    : QObject(parent)
    , m_compositor(compositor)
{
}

ViewporterManager::~ViewporterManager()
{
}

bool ViewporterManager::initialize()
{
    if (!m_compositor) {
        return false;
    }
    
    // Verbinde das Signal für neue Oberflächen
    connect(m_compositor, &QWaylandCompositor::surfaceCreated, this, [this](QWaylandSurface* surface) {
        // Standardmäßig hat die Oberfläche keinen benutzerdefinierten Viewport
        m_surfaceViewports.insert(surface, QSize(-1, -1));
        
        // Verbinde das Signal zum Entfernen der Oberfläche
        connect(surface, &QWaylandSurface::surfaceDestroyed, this, [this, surface]() {
            m_surfaceViewports.remove(surface);
        });
    });
    
    return true;
}

void ViewporterManager::setViewportDestination(QWaylandSurface* surface, const QSize& destinationSize)
{
    if (!surface) {
        return;
    }
    
    // Speichere die Zielgröße für den Viewport
    m_surfaceViewports[surface] = destinationSize;
    
    // Signalisiere die Änderung
    emit viewportChanged(surface, destinationSize);
}

QSize ViewporterManager::getViewportDestination(QWaylandSurface* surface) const
{
    if (!surface || !m_surfaceViewports.contains(surface)) {
        return QSize(-1, -1); // Standardwert, wenn kein Viewport festgelegt ist
    }
    
    return m_surfaceViewports.value(surface);
}

} // namespace Wayland
} // namespace VivoX

// Include the Q_OBJECT-generated code for the nested classes
#include "WaylandProtocols.moc"
