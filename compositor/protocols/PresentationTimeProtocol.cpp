/**
 * @file PresentationTimeProtocol.cpp
 * @brief Implementierung des Presentation-Time-Protokolls
 * 
 * Diese Datei enthält die Implementierung des Presentation-Time-Protokolls, das für
 * die präzise Zeitsteuerung der Darstellung von Frames verantwortlich ist.
 */

 #include "PresentationTimeProtocol.h"
 #include "../core/Logger.h"
 
 // Wayland Presentation-Time-Protokoll Header
 #include <wayland-server-protocol.h>
 #include <wayland-server.h>
 #include "qwayland-server-presentation-time.h"
 
 namespace VivoX {
 namespace Wayland {
 
 // ===============================
 // PresentationFeedback Implementation
 // ===============================
 
 PresentationFeedback::PresentationFeedback(QWaylandSurface* surface, QWaylandResource resource, QObject* parent)
     : QObject(parent)
     , m_surface(surface)
     , m_resource(resource)
 {
     Core::Logger::instance().debug(
         QString("PresentationFeedback erstellt für Surface %1")
             .arg(reinterpret_cast<quintptr>(surface)),
         "Wayland"
     );
     
     // Verbinde Signale der Oberfläche
     connect(surface, &QWaylandSurface::destroyed, this, &PresentationFeedback::destroyed);
 }
 
 PresentationFeedback::~PresentationFeedback()
 {
     Core::Logger::instance().debug(
         QString("PresentationFeedback zerstört für Surface %1")
             .arg(reinterpret_cast<quintptr>(m_surface)),
         "Wayland"
     );
 }
 
 QWaylandSurface* PresentationFeedback::surface() const
 {
     return m_surface;
 }
 
 QWaylandResource PresentationFeedback::resource() const
 {
     return m_resource;
 }
 
 void PresentationFeedback::sendPresented(uint32_t tvSecHi, uint32_t tvSecLo, uint32_t tvNsec, 
                                        uint32_t refresh, uint32_t sequenceHigh, 
                                        uint32_t sequenceLow, uint32_t flags)
 {
     if (!m_resource.isNull()) {
         struct ::wl_resource* resource = m_resource.resource();
         wp_presentation_feedback_send_presented(resource, tvSecHi, tvSecLo, tvNsec, 
                                                refresh, sequenceHigh, sequenceLow, flags);
         
         Core::Logger::instance().debug(
             QString("PresentationFeedback: Presented gesendet für Surface %1 (time: %2.%3s, refresh: %4ns, seq: %5:%6, flags: %7)")
                 .arg(reinterpret_cast<quintptr>(m_surface))
                 .arg(static_cast<uint64_t>(tvSecHi) << 32 | tvSecLo)
                 .arg(tvNsec)
                 .arg(refresh)
                 .arg(sequenceHigh)
                 .arg(sequenceLow)
                 .arg(flags),
             "Wayland"
         );
         
         // Nach dem Senden des Presented-Events wird das Feedback-Objekt zerstört
         emit destroyed();
     } else {
         Core::Logger::instance().warning(
             QString("PresentationFeedback: Kann Presented nicht senden für Surface %1, Resource ist ungültig")
                 .arg(reinterpret_cast<quintptr>(m_surface)),
             "Wayland"
         );
     }
 }
 
 void PresentationFeedback::sendDiscarded()
 {
     if (!m_resource.isNull()) {
         struct ::wl_resource* resource = m_resource.resource();
         wp_presentation_feedback_send_discarded(resource);
         
         Core::Logger::instance().debug(
             QString("PresentationFeedback: Discarded gesendet für Surface %1")
                 .arg(reinterpret_cast<quintptr>(m_surface)),
             "Wayland"
         );
         
         // Nach dem Senden des Discarded-Events wird das Feedback-Objekt zerstört
         emit destroyed();
     } else {
         Core::Logger::instance().warning(
             QString("PresentationFeedback: Kann Discarded nicht senden für Surface %1, Resource ist ungültig")
                 .arg(reinterpret_cast<quintptr>(m_surface)),
             "Wayland"
         );
     }
 }
 
 // ===============================
 // PresentationTimeProtocol Implementation
 // ===============================
 
 // Wayland-Protokoll-Funktionen
 static void handle_destroy(wl_client *client, wl_resource *resource)
 {
     Q_UNUSED(client);
     wl_resource_destroy(resource);
 }
 
 static void handle_feedback(wl_client *client, wl_resource *resource, wl_resource *surface, uint32_t callback)
 {
     Q_UNUSED(client);
     
     // Hole das PresentationTimeProtocol-Objekt
     PresentationTimeProtocol *protocol = static_cast<PresentationTimeProtocol*>(wl_resource_get_user_data(resource));
     if (!protocol) {
         Core::Logger::instance().error("PresentationTimeProtocol: Ungültiges Protokoll-Objekt", "Wayland");
         return;
     }
     
     // Hole das QWaylandSurface-Objekt für die übergebene Wayland-Oberfläche
     QWaylandSurface *qSurface = QWaylandSurface::fromWlSurface(reinterpret_cast<::wl_surface*>(surface));
     if (!qSurface) {
         Core::Logger::instance().error("PresentationTimeProtocol: Ungültige Oberfläche", "Wayland");
         return;
     }
     
     // Erstelle eine neue Feedback-Ressource
     uint32_t version = wl_resource_get_version(resource);
     struct wl_resource *feedbackResource = wl_resource_create(client, &wp_presentation_feedback_interface, version, callback);
     if (!feedbackResource) {
         Core::Logger::instance().error("PresentationTimeProtocol: Konnte Feedback-Ressource nicht erstellen", "Wayland");
         wl_resource_post_no_memory(resource);
         return;
     }
     
     // Setze die Destruktor-Funktion für die Ressource
     wl_resource_set_implementation(feedbackResource, nullptr, protocol, nullptr);
     
     // Erstelle ein neues Feedback-Objekt
     protocol->createFeedback(qSurface, QWaylandResource(feedbackResource));
 }
 
 static const struct wp_presentation_interface presentation_interface = {
     handle_destroy,
     handle_feedback
 };
 
 PresentationTimeProtocol::PresentationTimeProtocol(QWaylandCompositor* compositor, QObject* parent)
     : QWaylandGlobal(compositor, 1, "wp_presentation", parent)
     , m_feedbacks()
 {
     Core::Logger::instance().info("PresentationTimeProtocol erstellt", "Wayland");
 }
 
 PresentationTimeProtocol::~PresentationTimeProtocol()
 {
     // Entferne alle Feedback-Objekte
     while (!m_feedbacks.isEmpty()) {
         removeFeedback(m_feedbacks.values().first());
     }
     
     Core::Logger::instance().info("PresentationTimeProtocol zerstört", "Wayland");
 }
 
 PresentationFeedback* PresentationTimeProtocol::createFeedback(QWaylandSurface* surface, QWaylandResource resource)
 {
     // Erstelle ein neues Feedback-Objekt
     PresentationFeedback* feedback = new PresentationFeedback(surface, resource, this);
     
     // Füge das Feedback-Objekt zur Liste hinzu
     m_feedbacks.insert(surface, feedback);
     
     // Verbinde Signale
     connect(feedback, &PresentationFeedback::destroyed, this, [this, feedback]() {
         removeFeedback(feedback);
     });
     
     // Sende Signal, dass ein neues Feedback-Objekt erstellt wurde
     emit feedbackCreated(feedback);
     
     Core::Logger::instance().info(
         QString("PresentationFeedback erstellt für Surface %1")
             .arg(reinterpret_cast<quintptr>(surface)),
         "Wayland"
     );
     
     return feedback;
 }
 
 void PresentationTimeProtocol::removeFeedback(PresentationFeedback* feedback)
 {
     if (m_feedbacks.values().contains(feedback)) {
         // Entferne das Feedback-Objekt aus der Liste
         m_feedbacks.remove(feedback->surface(), feedback);
         
         // Sende Signal, dass das Feedback-Objekt zerstört wurde
         emit feedbackDestroyed(feedback);
         
         Core::Logger::instance().info(
             QString("PresentationFeedback entfernt für Surface %1")
                 .arg(reinterpret_cast<quintptr>(feedback->surface())),
             "Wayland"
         );
         
         // Lösche das Feedback-Objekt
         feedback->deleteLater();
     }
 }
 
 QList<PresentationFeedback*> PresentationTimeProtocol::feedbacksForSurface(QWaylandSurface* surface) const
 {
     return m_feedbacks.values(surface);
 }
 
 void PresentationTimeProtocol::resourceCreated(QWaylandClient* client, uint id, int version)
 {
     // Erstelle eine neue Ressource für den Client
     QWaylandResource resource = add(client, id, version);
     struct ::wl_resource* wlResource = resource.resource();
     
     // Setze die Implementierung für die Ressource
     wl_resource_set_implementation(wlResource, &presentation_interface, this, nullptr);
     
     Core::Logger::instance().debug(
         QString("PresentationTimeProtocol Ressource erstellt: Client=%1, ID=%2, Version=%3")
             .arg(client->processId())
             .arg(id)
             .arg(version),
         "Wayland"
     );
     
     // Sende die Möglichkeiten des Presentation-Time-Protokolls an den Client
     // Flags:
     // - WP_PRESENTATION_CAPABILITY_CLOCK_HW: Hardwareuhr-Unterstützung
     // - WP_PRESENTATION_CAPABILITY_ZERO_COPY: Zero-Copy-Unterstützung
     wp_presentation_send_clock_id(wlResource, CLOCK_MONOTONIC);
     
     // Sende die unterstützten Funktionen
     uint32_t capabilities = WP_PRESENTATION_CAPABILITY_CLOCK_HW;
     if (compositor()->supportsEglStreamConsumer()) {
         capabilities |= WP_PRESENTATION_CAPABILITY_ZERO_COPY;
     }
     
     wp_presentation_send_capabilities(wlResource, capabilities);
 }
 
 // Hilfsfunktion zum Aufteilen eines 64-Bit-Zeitwerts in zwei 32-Bit-Werte
 void splitTimeValue(uint64_t time, uint32_t& high, uint32_t& low)
 {
     high = static_cast<uint32_t>((time >> 32) & 0xFFFFFFFF);
     low = static_cast<uint32_t>(time & 0xFFFFFFFF);
 }
 
 void PresentationTimeProtocol::presentSurface(QWaylandSurface* surface, const QWaylandSyncPoint& syncPoint, uint64_t frameTime)
 {
     // Prüfe, ob es Feedback-Objekte für diese Oberfläche gibt
     QList<PresentationFeedback*> feedbacks = feedbacksForSurface(surface);
     if (feedbacks.isEmpty()) {
         return;
     }
     
     // Aktuellen Zeitpunkt ermitteln
     struct timespec now;
     clock_gettime(CLOCK_MONOTONIC, &now);
     
     // Zeitwerte vorbereiten
     uint32_t tvSecHi, tvSecLo;
     uint32_t tvNsec = now.tv_nsec;
     splitTimeValue(now.tv_sec, tvSecHi, tvSecLo);
     
     // Aktualisierungsfrequenz des Displays berechnen
     // Wir nehmen hier eine Standard-Bildschirmfrequenz von 60Hz an
     // In einer realen Implementierung würde man diese Informationen aus dem Display-System bekommen
     uint32_t refresh = 16666667; // 60 Hz in Nanosekunden
     
     // Sequenznummer aus dem Sync-Punkt extrahieren
     uint32_t sequenceHigh = 0;
     uint32_t sequenceLow = syncPoint.frame();
     
     // Flags für die Präsentation
     // - WP_PRESENTATION_FEEDBACK_KIND_VSYNC: Frame wurde mit VSync gerendert
     // - WP_PRESENTATION_FEEDBACK_KIND_HW_CLOCK: Hardware-Uhr wurde verwendet
     // - WP_PRESENTATION_FEEDBACK_KIND_HW_COMPLETION: Hardware hat die Fertigstellung gemeldet
     // - WP_PRESENTATION_FEEDBACK_KIND_ZERO_COPY: Zero-Copy wurde verwendet
     uint32_t flags = WP_PRESENTATION_FEEDBACK_KIND_VSYNC | WP_PRESENTATION_FEEDBACK_KIND_HW_CLOCK;
     
     if (frameTime > 0) {
         // Wenn ein frameTime angegeben wurde, verwenden wir diese statt der aktuellen Zeit
         splitTimeValue(frameTime / 1000000000, tvSecHi, tvSecLo); // Umrechnung von ns in s
         tvNsec = frameTime % 1000000000; // Restliche ns
     }
     
     // Sende die Präsentations-Rückmeldungen
     for (PresentationFeedback* feedback : feedbacks) {
         feedback->sendPresented(tvSecHi, tvSecLo, tvNsec, refresh, sequenceHigh, sequenceLow, flags);
     }
 }
 
 void PresentationTimeProtocol::discardSurface(QWaylandSurface* surface)
 {
     // Prüfe, ob es Feedback-Objekte für diese Oberfläche gibt
     QList<PresentationFeedback*> feedbacks = feedbacksForSurface(surface);
     if (feedbacks.isEmpty()) {
         return;
     }
     
     // Sende die Discard-Rückmeldungen
     for (PresentationFeedback* feedback : feedbacks) {
         feedback->sendDiscarded();
     }
 }
 
 } // namespace Wayland
 } // namespace VivoX
 