/**
 * @file WaylandLayerShell.cpp
 * @brief Implementierung des wlr-layer-shell-Protokolls
 * 
 * Diese Datei enthält die Implementierung des wlr-layer-shell-Protokolls, das für
 * die Erstellung von Oberflächen in verschiedenen Ebenen (Layers) verantwortlich ist.
 * Dies ist besonders wichtig für Panels, Docks und andere UI-Elemente, die über oder
 * unter normalen Fenstern angezeigt werden sollen.
 */

 #include "WaylandLayerShell.h"
 #include "../core/Logger.h"
 #include <QWaylandCompositorExtensionTemplate>
 #include <private/qwaylandcompositor_p.h>
 #include <private/qwaylandsurface_p.h>
 #include <QWaylandQuickShellSurfaceItem>
 
 // Die folgenden Includes würden in einer realen Implementierung benötigt
 // Wir definieren hier die Stub-Protokoll-Header
 #include <wayland-server.h>
 
 // Externe Protokoll-Header für zwlr_layer_shell_v1
 extern "C" {
 #include "protocol/wlr-layer-shell-unstable-v1-protocol.h"
 }
 
 namespace VivoX {
 namespace Wayland {
 
 // ===============================
 // LayerSurface Implementation
 // ===============================
 
 LayerSurface::LayerSurface(QWaylandSurface* surface, QWaylandResource resource, 
                          LayerShellLayer layer, const QString& nameSpace, QObject* parent)
     : QObject(parent)
     , m_surface(surface)
     , m_resource(resource)
     , m_layer(layer)
     , m_anchor(AnchorNone)
     , m_exclusiveZone(0)
     , m_margins(0, 0, 0, 0)
     , m_desiredSize(0, 0)
     , m_namespace(nameSpace)
 {
     // Verbinde Signale der Oberfläche
     connect(surface, &QWaylandSurface::destroyed, this, &LayerSurface::surfaceDestroyed);
     
     Core::Logger::instance().debug(
         QString("LayerSurface erstellt: Namespace=%1, Layer=%2")
             .arg(nameSpace)
             .arg(layer),
         "Wayland"
     );
 }
 
 LayerSurface::~LayerSurface()
 {
     Core::Logger::instance().debug(
         QString("LayerSurface zerstört: Namespace=%1")
             .arg(m_namespace),
         "Wayland"
     );
 }
 
 QWaylandSurface* LayerSurface::surface() const
 {
     return m_surface;
 }
 
 LayerShellLayer LayerSurface::layer() const
 {
     return m_layer;
 }
 
 void LayerSurface::setLayer(LayerShellLayer layer)
 {
     if (m_layer != layer) {
         m_layer = layer;
         emit layerChanged(layer);
         
         Core::Logger::instance().debug(
             QString("LayerSurface Layer geändert: Namespace=%1, Layer=%2")
                 .arg(m_namespace)
                 .arg(layer),
             "Wayland"
         );
     }
 }
 
 uint LayerSurface::anchor() const
 {
     return m_anchor;
 }
 
 void LayerSurface::setAnchor(uint anchor)
 {
     if (m_anchor != anchor) {
         m_anchor = anchor;
         emit anchorChanged(anchor);
         
         Core::Logger::instance().debug(
             QString("LayerSurface Anker geändert: Namespace=%1, Anker=%2")
                 .arg(m_namespace)
                 .arg(anchor),
             "Wayland"
         );
     }
 }
 
 int LayerSurface::exclusiveZone() const
 {
     return m_exclusiveZone;
 }
 
 void LayerSurface::setExclusiveZone(int exclusiveZone)
 {
     if (m_exclusiveZone != exclusiveZone) {
         m_exclusiveZone = exclusiveZone;
         emit exclusiveZoneChanged(exclusiveZone);
         
         Core::Logger::instance().debug(
             QString("LayerSurface ExclusiveZone geändert: Namespace=%1, ExclusiveZone=%2")
                 .arg(m_namespace)
                 .arg(exclusiveZone),
             "Wayland"
         );
     }
 }
 
 QMargins LayerSurface::margins() const
 {
     return m_margins;
 }
 
 void LayerSurface::setMargins(const QMargins& margins)
 {
     if (m_margins != margins) {
         m_margins = margins;
         emit marginsChanged(margins);
         
         Core::Logger::instance().debug(
             QString("LayerSurface Margins geändert: Namespace=%1, Margins=(%2,%3,%4,%5)")
                 .arg(m_namespace)
                 .arg(margins.left())
                 .arg(margins.top())
                 .arg(margins.right())
                 .arg(margins.bottom()),
             "Wayland"
         );
     }
 }
 
 QSize LayerSurface::desiredSize() const
 {
     return m_desiredSize;
 }
 
 void LayerSurface::setDesiredSize(const QSize& size)
 {
     if (m_desiredSize != size) {
         m_desiredSize = size;
         emit desiredSizeChanged(size);
         
         Core::Logger::instance().debug(
             QString("LayerSurface DesiredSize geändert: Namespace=%1, Size=(%2,%3)")
                 .arg(m_namespace)
                 .arg(size.width())
                 .arg(size.height()),
             "Wayland"
         );
     }
 }
 
 QString LayerSurface::getNamespace() const
 {
     return m_namespace;
 }
 
 QWaylandResource LayerSurface::resource() const
 {
     return m_resource;
 }
 
 // ===============================
 // WaylandLayerShell Implementation
 // ===============================
 
 // Protokoll-Handler-Funktionen
 // Diese Funktionen werden aufgerufen, wenn ein Client bestimmte Requests sendet
 
 static void layer_shell_get_layer_surface(wl_client *client, wl_resource *resource,
                                         uint32_t id, wl_resource *surface_resource,
                                         wl_resource *output_resource, uint32_t layer,
                                         const char *namespace_str)
 {
     // Holen Sie sich das WaylandLayerShell-Objekt aus dem Ressourcen-Benutzerdaten
     WaylandLayerShell *shell = static_cast<WaylandLayerShell *>(wl_resource_get_user_data(resource));
 
     // Holen Sie sich die QWaylandSurface aus der Wayland-Ressource
     QWaylandSurface *surface = QWaylandSurface::fromWlSurface(
         wl_resource_get_user_data(surface_resource));
 
     if (!surface) {
         wl_resource_post_error(resource, WL_DISPLAY_ERROR_INVALID_OBJECT,
                               "Ungültige Oberfläche für layer_surface");
         return;
     }
 
     // Überprüfe, ob layer gültig ist
     if (layer > static_cast<uint32_t>(LayerShellLayer::OverlayLayer)) {
         wl_resource_post_error(resource, ZWLR_LAYER_SHELL_V1_ERROR_INVALID_LAYER,
                               "Ungültige Layer-Ebene: %d", layer);
         return;
     }
 
     // Erstelle eine neue Ressource für die Layer-Oberfläche
     uint32_t version = wl_resource_get_version(resource);
     wl_resource *layer_surface_resource = wl_resource_create(client, 
                                                           &zwlr_layer_surface_v1_interface,
                                                           version, id);
     if (!layer_surface_resource) {
         wl_client_post_no_memory(client);
         return;
     }
 
     // Erstellen Sie das LayerSurface-Objekt
     QWaylandResource qresource(layer_surface_resource);
     QString nameSpace = QString::fromUtf8(namespace_str);
     LayerShellLayer layerEnum = static_cast<LayerShellLayer>(layer);
     
     shell->handleGetLayerSurface(surface, qresource, layerEnum, nameSpace);
 }
 
 static void layer_shell_destroy(wl_client *client, wl_resource *resource)
 {
     Q_UNUSED(client);
     wl_resource_destroy(resource);
 }
 
 // Layer-Shell-Protokoll-Interface
 static const struct zwlr_layer_shell_v1_interface layer_shell_implementation = {
     layer_shell_get_layer_surface,
     layer_shell_destroy
 };
 
 // Layer-Surface-Handler-Funktionen
 static void layer_surface_set_size(wl_client *client, wl_resource *resource,
                                  uint32_t width, uint32_t height)
 {
     Q_UNUSED(client);
     
     // Holen Sie sich das LayerSurface-Objekt aus dem Ressourcen-Benutzerdaten
     LayerSurface *layerSurface = static_cast<LayerSurface *>(wl_resource_get_user_data(resource));
     if (!layerSurface)
         return;
 
     layerSurface->setDesiredSize(QSize(width, height));
 }
 
 static void layer_surface_set_anchor(wl_client *client, wl_resource *resource, uint32_t anchor)
 {
     Q_UNUSED(client);
     
     // Überprüfe, ob anchor gültig ist
     if ((anchor & ~(ZWLR_LAYER_SURFACE_V1_ANCHOR_TOP | ZWLR_LAYER_SURFACE_V1_ANCHOR_BOTTOM |
                    ZWLR_LAYER_SURFACE_V1_ANCHOR_LEFT | ZWLR_LAYER_SURFACE_V1_ANCHOR_RIGHT)) != 0) {
         wl_resource_post_error(resource, ZWLR_LAYER_SURFACE_V1_ERROR_INVALID_ANCHOR,
                               "Ungültiger Anker: %d", anchor);
         return;
     }
 
     // Holen Sie sich das LayerSurface-Objekt aus dem Ressourcen-Benutzerdaten
     LayerSurface *layerSurface = static_cast<LayerSurface *>(wl_resource_get_user_data(resource));
     if (!layerSurface)
         return;
 
     layerSurface->setAnchor(anchor);
 }
 
 static void layer_surface_set_exclusive_zone(wl_client *client, wl_resource *resource, int32_t zone)
 {
     Q_UNUSED(client);
     
     // Holen Sie sich das LayerSurface-Objekt aus dem Ressourcen-Benutzerdaten
     LayerSurface *layerSurface = static_cast<LayerSurface *>(wl_resource_get_user_data(resource));
     if (!layerSurface)
         return;
 
     layerSurface->setExclusiveZone(zone);
 }
 
 static void layer_surface_set_margin(wl_client *client, wl_resource *resource,
                                    int32_t top, int32_t right, int32_t bottom, int32_t left)
 {
     Q_UNUSED(client);
     
     // Holen Sie sich das LayerSurface-Objekt aus dem Ressourcen-Benutzerdaten
     LayerSurface *layerSurface = static_cast<LayerSurface *>(wl_resource_get_user_data(resource));
     if (!layerSurface)
         return;
 
     layerSurface->setMargins(QMargins(left, top, right, bottom));
 }
 
 static void layer_surface_set_keyboard_interactivity(wl_client *client, wl_resource *resource,
                                                   uint32_t keyboard_interactivity)
 {
     Q_UNUSED(client);
     Q_UNUSED(resource);
     Q_UNUSED(keyboard_interactivity);
     
     // In dieser Implementierung nicht benötigt, da wir keine Tastaturinteraktivität unterstützen
     // Könnte in einer erweiterten Version implementiert werden
 }
 
 static void layer_surface_get_popup(wl_client *client, wl_resource *resource, wl_resource *popup_resource)
 {
     Q_UNUSED(client);
     Q_UNUSED(resource);
     Q_UNUSED(popup_resource);
     
     // In dieser Implementierung nicht benötigt, da wir keine Popup-Unterstützung haben
     // Könnte in einer erweiterten Version implementiert werden
 }
 
 static void layer_surface_ack_configure(wl_client *client, wl_resource *resource, uint32_t serial)
 {
     Q_UNUSED(client);
     Q_UNUSED(resource);
     Q_UNUSED(serial);
     
     // In dieser Implementierung nicht vollständig implementiert
     // Der Client bestätigt eine Konfiguration mit der angegebenen Seriennummer
     // In einer vollständigen Implementierung würden wir die Konfiguration als bestätigt markieren
 }
 
 static void layer_surface_destroy(wl_client *client, wl_resource *resource)
 {
     Q_UNUSED(client);
     wl_resource_destroy(resource);
 }
 
 static void layer_surface_set_layer(wl_client *client, wl_resource *resource, uint32_t layer)
 {
     Q_UNUSED(client);
     
     // Überprüfe, ob layer gültig ist
     if (layer > static_cast<uint32_t>(LayerShellLayer::OverlayLayer)) {
         wl_resource_post_error(resource, ZWLR_LAYER_SURFACE_V1_ERROR_INVALID_LAYER,
                               "Ungültige Layer-Ebene: %d", layer);
         return;
     }
 
     // Holen Sie sich das LayerSurface-Objekt aus dem Ressourcen-Benutzerdaten
     LayerSurface *layerSurface = static_cast<LayerSurface *>(wl_resource_get_user_data(resource));
     if (!layerSurface)
         return;
 
     layerSurface->setLayer(static_cast<LayerShellLayer>(layer));
 }
 
 // Layer-Surface-Protokoll-Interface
 static const struct zwlr_layer_surface_v1_interface layer_surface_implementation = {
     layer_surface_destroy,
     layer_surface_set_size,
     layer_surface_set_anchor,
     layer_surface_set_exclusive_zone,
     layer_surface_set_margin,
     layer_surface_set_keyboard_interactivity,
     layer_surface_get_popup,
     layer_surface_ack_configure,
     layer_surface_set_layer
 };
 
 // Resource-Destroy-Callback für Layer-Surface
 static void layer_surface_resource_destroy(wl_resource *resource)
 {
     LayerSurface *layerSurface = static_cast<LayerSurface *>(wl_resource_get_user_data(resource));
     if (layerSurface) {
         // Die LayerSurface wird durch die Verbindung zum surfaceDestroyed-Signal automatisch entfernt
         // Wenn die Oberfläche jedoch noch existiert, müssen wir das surfaceDestroyed-Signal manuell auslösen
         emit layerSurface->surfaceDestroyed();
     }
 }
 
 WaylandLayerShell::WaylandLayerShell(QWaylandCompositor* compositor, QObject* parent)
     : QWaylandGlobal(compositor, 1, "zwlr_layer_shell_v1", parent)
     , m_layerSurfaces()
 {
     Core::Logger::instance().info("WaylandLayerShell erstellt", "Wayland");
 }
 
 WaylandLayerShell::~WaylandLayerShell()
 {
     // Entferne alle Layer-Shell-Oberflächen
     while (!m_layerSurfaces.isEmpty()) {
         removeLayerSurface(m_layerSurfaces.first());
     }
     
     Core::Logger::instance().info("WaylandLayerShell zerstört", "Wayland");
 }
 
 QList<LayerSurface*> WaylandLayerShell::layerSurfaces() const
 {
     return m_layerSurfaces;
 }
 
 QList<LayerSurface*> WaylandLayerShell::layerSurfacesForLayer(LayerShellLayer layer) const
 {
     QList<LayerSurface*> surfaces;
     
     for (LayerSurface* surface : m_layerSurfaces) {
         if (surface->layer() == layer) {
             surfaces.append(surface);
         }
     }
     
     return surfaces;
 }
 
 void WaylandLayerShell::resourceCreated(QWaylandClient* client, uint id, int version)
 {
     // Erstelle eine neue wl_resource für den Client
     wl_resource *resource = wl_resource_create(client->client(), &zwlr_layer_shell_v1_interface, 
                                              version, id);
     if (!resource) {
         wl_client_post_no_memory(client->client());
         return;
     }
 
     // Setze die Implementation und den Destruktor
     wl_resource_set_implementation(resource, &layer_shell_implementation, this, nullptr);
     
     Core::Logger::instance().debug(
         QString("WaylandLayerShell Ressource erstellt: Client=%1, ID=%2, Version=%3")
             .arg(client->processId())
             .arg(id)
             .arg(version),
         "Wayland"
     );
 }
 
 void WaylandLayerShell::handleGetLayerSurface(QWaylandSurface* surface, QWaylandResource resource, 
                                             LayerShellLayer layer, const QString& nameSpace)
 {
     // Erstelle eine neue Layer-Shell-Oberfläche
     LayerSurface* layerSurface = createLayerSurface(surface, resource, layer, nameSpace);
     
     // Setze die Implementation und den Destruktor für die Layer-Surface-Ressource
     wl_resource *wl_resource = resource.resource();
     wl_resource_set_implementation(wl_resource, &layer_surface_implementation, 
                                  layerSurface, layer_surface_resource_destroy);
     
     // Sende die erste Konfiguration
     sendConfigure(layerSurface, 0);
 }
 
 void WaylandLayerShell::sendConfigure(LayerSurface* layerSurface, uint32_t serial)
 {
     wl_resource *resource = layerSurface->resource().resource();
     
     // Sende configure-Event an den Client
     // width und height sind die gewünschte Größe des Compositors für die Oberfläche
     // Wenn 0, kann der Client die Größe selbst wählen
     uint32_t width = layerSurface->desiredSize().width();
     uint32_t height = layerSurface->desiredSize().height();
     
     zwlr_layer_surface_v1_send_configure(resource, serial, width, height);
 }
 
 LayerSurface* WaylandLayerShell::createLayerSurface(QWaylandSurface* surface, QWaylandResource resource, 
                                                   LayerShellLayer layer, const QString& nameSpace)
 {
     // Erstelle eine neue Layer-Shell-Oberfläche
     LayerSurface* layerSurface = new LayerSurface(surface, resource, layer, nameSpace, this);
     
     // Füge die Oberfläche zur Liste hinzu
     m_layerSurfaces.append(layerSurface);
     
     // Verbinde Signale
     connect(layerSurface, &LayerSurface::surfaceDestroyed, this, [this, layerSurface]() {
         removeLayerSurface(layerSurface);
     });
     
     // Sende Signal, dass eine neue Layer-Shell-Oberfläche erstellt wurde
     emit layerSurfaceCreated(layerSurface);
     
     Core::Logger::instance().info(
         QString("LayerSurface erstellt: Namespace=%1, Layer=%2")
             .arg(nameSpace)
             .arg(layer),
         "Wayland"
     );
     
     return layerSurface;
 }
 
 void WaylandLayerShell::removeLayerSurface(LayerSurface* layerSurface)
 {
     if (m_layerSurfaces.contains(layerSurface)) {
         // Entferne die Oberfläche aus der Liste
         m_layerSurfaces.removeOne(layerSurface);
         
         // Sende Signal, dass die Layer-Shell-Oberfläche zerstört wurde
         emit layerSurfaceDestroyed(layerSurface);
         
         Core::Logger::instance().info(
             QString("LayerSurface entfernt: Namespace=%1")
                 .arg(layerSurface->getNamespace()),
             "Wayland"
         );
         
         // Lösche die Oberfläche
         layerSurface->deleteLater();
     }
 }
 
 // Öffentliche Hilfsfunktionen
 
 void WaylandLayerShell::configureSurface(LayerSurface* layerSurface, const QSize& size)
 {
     if (!layerSurface || !m_layerSurfaces.contains(layerSurface))
         return;
     
     // Setze die gewünschte Größe
     layerSurface->setDesiredSize(size);
     
     // Sende eine neue Konfiguration mit inkrementierter Seriennummer
     static uint32_t serial = 0;
     sendConfigure(layerSurface, ++serial);
 }
 
 void WaylandLayerShell::configureSurfaces(LayerShellLayer layer, const QSize& size)
 {
     for (LayerSurface* surface : layerSurfacesForLayer(layer)) {
         configureSurface(surface, size);
     }
 }
 
 QList<LayerSurface*> WaylandLayerShell::layerSurfacesForNamespace(const QString& nameSpace) const
 {
     QList<LayerSurface*> surfaces;
     
     for (LayerSurface* surface : m_layerSurfaces) {
         if (surface->getNamespace() == nameSpace) {
             surfaces.append(surface);
         }
     }
     
     return surfaces;
 }
 
 LayerSurface* WaylandLayerShell::findLayerSurface(QWaylandSurface* surface) const
 {
     for (LayerSurface* layerSurface : m_layerSurfaces) {
         if (layerSurface->surface() == surface) {
             return layerSurface;
         }
     }
     
     return nullptr;
 }
 
 } // namespace Wayland
 } // namespace VivoX
 