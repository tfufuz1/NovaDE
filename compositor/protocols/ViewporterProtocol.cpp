/**
 * @file ViewporterProtocol.cpp
 * @brief Implementierung des Viewporter-Protokolls
 * 
 * Diese Datei enthält die Implementierung des Viewporter-Protokolls, das für
 * die Skalierung und Beschneidung von Oberflächen verantwortlich ist.
 */

 #include "ViewporterProtocol.h"
 #include "../core/Logger.h"
 #include <QtWaylandCompositor/private/qwaylandutils_p.h>
 #include <wayland-server-protocol.h>
 #include <wayland-viewporter-server-protocol.h>
 
 namespace VivoX {
 namespace Wayland {
 
 // ===============================
 // Viewport Implementation
 // ===============================
 
 Viewport::Viewport(QWaylandSurface* surface, QWaylandResource resource, QObject* parent)
     : QObject(parent)
     , m_surface(surface)
     , m_resource(resource)
     , m_sourceRect()
     , m_destinationSize()
 {
     // Verbinde Signale der Oberfläche
     connect(surface, &QWaylandSurface::destroyed, this, &Viewport::viewportDestroyed);
     
     Core::Logger::instance().debug(
         QString("Viewport erstellt für Surface %1")
             .arg(reinterpret_cast<quintptr>(surface)),
         "Wayland"
     );
 }
 
 Viewport::~Viewport()
 {
     Core::Logger::instance().debug(
         QString("Viewport zerstört für Surface %1")
             .arg(reinterpret_cast<quintptr>(m_surface)),
         "Wayland"
     );
     
     if (m_resource.resource()) {
         wl_resource_destroy(m_resource.resource());
     }
 }
 
 QWaylandSurface* Viewport::surface() const
 {
     return m_surface;
 }
 
 QWaylandResource Viewport::resource() const
 {
     return m_resource;
 }
 
 QRectF Viewport::sourceRect() const
 {
     return m_sourceRect;
 }
 
 void Viewport::setSourceRect(const QRectF& rect)
 {
     if (m_sourceRect != rect) {
         m_sourceRect = rect;
         emit sourceRectChanged(rect);
         
         Core::Logger::instance().debug(
             QString("Viewport SourceRect geändert: Surface=%1, Rect=(%2,%3,%4,%5)")
                 .arg(reinterpret_cast<quintptr>(m_surface))
                 .arg(rect.x())
                 .arg(rect.y())
                 .arg(rect.width())
                 .arg(rect.height()),
             "Wayland"
         );
     }
 }
 
 QSizeF Viewport::destinationSize() const
 {
     return m_destinationSize;
 }
 
 void Viewport::setDestinationSize(const QSizeF& size)
 {
     if (m_destinationSize != size) {
         m_destinationSize = size;
         emit destinationSizeChanged(size);
         
         Core::Logger::instance().debug(
             QString("Viewport DestinationSize geändert: Surface=%1, Size=(%2,%3)")
                 .arg(reinterpret_cast<quintptr>(m_surface))
                 .arg(size.width())
                 .arg(size.height()),
             "Wayland"
         );
     }
 }
 
 // ===============================
 // ViewporterProtocol Implementation
 // ===============================
 
 // Statische Callback-Funktionen für das Wayland-Protokoll
 static void destroy_viewport(wl_client *client, wl_resource *resource)
 {
     Q_UNUSED(client);
     wl_resource_destroy(resource);
 }
 
 static void set_source(wl_client *client, wl_resource *resource, wl_fixed_t x, wl_fixed_t y, wl_fixed_t width, wl_fixed_t height)
 {
     Q_UNUSED(client);
     Viewport *viewport = static_cast<Viewport*>(wl_resource_get_user_data(resource));
     
     // Konvertiere wl_fixed_t zu double
     double dx = wl_fixed_to_double(x);
     double dy = wl_fixed_to_double(y);
     double dw = wl_fixed_to_double(width);
     double dh = wl_fixed_to_double(height);
     
     // Prüfe, ob die Parameter gültig sind
     if (width == wl_fixed_from_int(-1) && height == wl_fixed_from_int(-1)) {
         // Spezialfall: -1,-1 bedeutet "zurücksetzen auf Standard" (gesamte Oberfläche)
         viewport->setSourceRect(QRectF());
     } else if (dw <= 0 || dh <= 0) {
         // Ungültige Werte
         wl_resource_post_error(resource, WP_VIEWPORT_ERROR_BAD_VALUE,
                             "invalid source dimensions: %fx%f", dw, dh);
         return;
     } else {
         viewport->setSourceRect(QRectF(dx, dy, dw, dh));
     }
 }
 
 static void set_destination(wl_client *client, wl_resource *resource, int32_t width, int32_t height)
 {
     Q_UNUSED(client);
     Viewport *viewport = static_cast<Viewport*>(wl_resource_get_user_data(resource));
     
     // Prüfe, ob die Parameter gültig sind
     if (width == -1 && height == -1) {
         // Spezialfall: -1,-1 bedeutet "zurücksetzen auf Standard" (Quelloberfläche)
         viewport->setDestinationSize(QSizeF());
     } else if (width <= 0 || height <= 0) {
         // Ungültige Werte
         wl_resource_post_error(resource, WP_VIEWPORT_ERROR_BAD_VALUE,
                             "invalid destination dimensions: %dx%d", width, height);
         return;
     } else {
         viewport->setDestinationSize(QSizeF(width, height));
     }
 }
 
 // wp_viewport Schnittstelle
 static const struct wp_viewport_interface viewport_interface = {
     destroy_viewport,
     set_source,
     set_destination
 };
 
 // Statische Callback-Funktionen für das wp_viewporter Protokoll
 static void destroy_viewporter(wl_client *client, wl_resource *resource)
 {
     Q_UNUSED(client);
     wl_resource_destroy(resource);
 }
 
 static void get_viewport(wl_client *client, wl_resource *resource, uint32_t id, wl_resource *surface_resource)
 {
     ViewporterProtocol *protocol = static_cast<ViewporterProtocol*>(wl_resource_get_user_data(resource));
     QWaylandSurface *surface = QWaylandSurface::fromResource(surface_resource);
     
     if (!surface) {
         wl_resource_post_error(resource, WP_VIEWPORTER_ERROR_VIEWPORT_EXISTS,
                             "the surface doesn't exist");
         return;
     }
     
     // Prüfe, ob bereits ein Viewport für diese Oberfläche existiert
     if (protocol->viewportForSurface(surface)) {
         wl_resource_post_error(resource, WP_VIEWPORTER_ERROR_VIEWPORT_EXISTS,
                             "a viewport for that surface already exists");
         return;
     }
     
     // Erstelle eine neue Ressource für das Viewport-Objekt
     wl_resource *viewport_resource = wl_resource_create(client,
                                                      &wp_viewport_interface,
                                                      wl_resource_get_version(resource),
                                                      id);
     
     if (!viewport_resource) {
         wl_client_post_no_memory(client);
         return;
     }
     
     // Erstelle ein neues Viewport-Objekt
     QWaylandResource qresource(viewport_resource);
     protocol->createViewport(surface, qresource);
     
     // Setze Benutzerdaten und Destruktor-Handler
     wl_resource_set_implementation(viewport_resource, &viewport_interface,
                                  qresource.resource() ? wl_resource_get_user_data(qresource.resource()) : nullptr,
                                  [](wl_resource *resource) {
                                      Viewport *viewport = static_cast<Viewport*>(wl_resource_get_user_data(resource));
                                      if (viewport) {
                                          emit viewport->viewportDestroyed();
                                      }
                                  });
 }
 
 // wp_viewporter Schnittstelle
 static const struct wp_viewporter_interface viewporter_interface = {
     destroy_viewporter,
     get_viewport
 };
 
 ViewporterProtocol::ViewporterProtocol(QWaylandCompositor* compositor, QObject* parent)
     : QWaylandGlobal(compositor, 1, "wp_viewporter", parent)
     , m_viewports()
 {
     Core::Logger::instance().info("ViewporterProtocol erstellt", "Wayland");
 }
 
 ViewporterProtocol::~ViewporterProtocol()
 {
     // Entferne alle Viewport-Objekte
     QList<Viewport*> viewports = m_viewports.values();
     for (Viewport* viewport : viewports) {
         removeViewport(viewport);
     }
     
     Core::Logger::instance().info("ViewporterProtocol zerstört", "Wayland");
 }
 
 Viewport* ViewporterProtocol::createViewport(QWaylandSurface* surface, QWaylandResource resource)
 {
     // Prüfe, ob bereits ein Viewport für diese Oberfläche existiert
     if (m_viewports.contains(surface)) {
         Core::Logger::instance().warning(
             QString("Viewport existiert bereits für Surface %1")
                 .arg(reinterpret_cast<quintptr>(surface)),
             "Wayland"
         );
         return m_viewports.value(surface);
     }
     
     // Erstelle ein neues Viewport-Objekt
     Viewport* viewport = new Viewport(surface, resource, this);
     
     // Füge das Viewport-Objekt zur Liste hinzu
     m_viewports.insert(surface, viewport);
     
     // Verbinde Signale
     connect(viewport, &Viewport::viewportDestroyed, this, [this, viewport]() {
         removeViewport(viewport);
     });
     
     // Verbinde Signal für Surface-Zerstörung
     connect(surface, &QWaylandSurface::destroyed, this, [this, surface]() {
         if (Viewport* viewport = m_viewports.value(surface, nullptr)) {
             removeViewport(viewport);
         }
     });
     
     // Setze Benutzer-Daten auf der Ressource
     if (resource.resource()) {
         wl_resource_set_user_data(resource.resource(), viewport);
     }
     
     // Sende Signal, dass ein neues Viewport-Objekt erstellt wurde
     emit viewportCreated(viewport);
     
     Core::Logger::instance().info(
         QString("Viewport erstellt für Surface %1")
             .arg(reinterpret_cast<quintptr>(surface)),
         "Wayland"
     );
     
     return viewport;
 }
 
 void ViewporterProtocol::removeViewport(Viewport* viewport)
 {
     if (viewport && m_viewports.values().contains(viewport)) {
         // Entferne das Viewport-Objekt aus der Liste
         m_viewports.remove(viewport->surface());
         
         // Sende Signal, dass das Viewport-Objekt zerstört wurde
         emit viewportDestroyed(viewport);
         
         Core::Logger::instance().info(
             QString("Viewport entfernt für Surface %1")
                 .arg(reinterpret_cast<quintptr>(viewport->surface())),
             "Wayland"
         );
         
         // Lösche das Viewport-Objekt
         viewport->deleteLater();
     }
 }
 
 Viewport* ViewporterProtocol::viewportForSurface(QWaylandSurface* surface) const
 {
     return m_viewports.value(surface, nullptr);
 }
 
 void ViewporterProtocol::resourceCreated(QWaylandClient* client, uint id, int version)
 {
     // Erstelle eine neue Ressource für den Client
     wl_resource *resource = wl_resource_create(client->client(), &wp_viewporter_interface, version, id);
     
     if (!resource) {
         wl_client_post_no_memory(client->client());
         return;
     }
     
     // Setze Implementierung und Benutzerdaten
     wl_resource_set_implementation(resource, &viewporter_interface, this, nullptr);
     
     Core::Logger::instance().debug(
         QString("ViewporterProtocol Ressource erstellt: Client=%1, ID=%2, Version=%3")
             .arg(client->processId())
             .arg(id)
             .arg(version),
         "Wayland"
     );
 }
 
 void ViewporterProtocol::bind(wl_client *client, uint32_t version, uint32_t id)
 {
     QWaylandClient *waylandClient = static_cast<QWaylandClient *>(QWaylandClient::fromWlClient(compositor(), client));
     resourceCreated(waylandClient, id, version);
 }
 
 } // namespace Wayland
 } // namespace VivoX
 