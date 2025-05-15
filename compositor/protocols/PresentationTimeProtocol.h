/**
 * @file PresentationTimeProtocol.h
 * @brief Implementierung des Presentation-Time-Protokolls
 *
 * Diese Datei enthält die Implementierung des Presentation-Time-Protokolls, das für
 * die präzise Zeitsteuerung der Darstellung von Frames verantwortlich ist.
 */

#pragma once

#include <QObject>
#include <QWaylandCompositor>
#include <QWaylandSurface>
#include <QWaylandResource>
#include <QWaylandGlobal>
#include <QWaylandSyncPoint>
#include <QMap>
#include <chrono>

// Wayland Presentation-Time-Protokoll Definitionen
#define WP_PRESENTATION_FEEDBACK_KIND_VSYNC 0x1
#define WP_PRESENTATION_FEEDBACK_KIND_HW_CLOCK 0x2
#define WP_PRESENTATION_FEEDBACK_KIND_HW_COMPLETION 0x4
#define WP_PRESENTATION_FEEDBACK_KIND_ZERO_COPY 0x8

#define WP_PRESENTATION_CAPABILITY_CLOCK_HW 0x1
#define WP_PRESENTATION_CAPABILITY_ZERO_COPY 0x2

namespace VivoX {
    namespace Wayland {

        class PresentationTimeProtocol; // Vorwärtsdeklaration

        /**
         * @class PresentationFeedback
         * @brief Repräsentiert ein Presentation-Feedback-Objekt
         *
         * Diese Klasse repräsentiert ein Presentation-Feedback-Objekt, das für die
         * Rückmeldung über die Darstellung eines Frames verwendet wird.
         */
        class PresentationFeedback : public QObject {
            Q_OBJECT

        public:
            /**
             * @brief Konstruktor
             * @param surface Wayland-Oberfläche
             * @param resource Wayland-Ressource
             * @param parent Elternobjekt
             */
            explicit PresentationFeedback(QWaylandSurface* surface, QWaylandResource resource, QObject* parent = nullptr);

            /**
             * @brief Destruktor
             */
            ~PresentationFeedback();

            /**
             * @brief Gibt die Wayland-Oberfläche zurück
             * @return Wayland-Oberfläche
             */
            QWaylandSurface* surface() const;

            /**
             * @brief Gibt die Wayland-Ressource zurück
             * @return Wayland-Ressource
             */
            QWaylandResource resource() const;

            /**
             * @brief Sendet eine Presented-Rückmeldung
             * @param tvSecHi Oberer Teil der Sekunden-Zeitkomponente
             * @param tvSecLo Unterer Teil der Sekunden-Zeitkomponente
             * @param tvNsec Nanosekunden-Zeitkomponente
             * @param refresh Aktualisierungsintervall des Displays in Nanosekunden
             * @param sequenceHigh Oberer Teil der Sequenznummer
             * @param sequenceLow Unterer Teil der Sequenznummer
             * @param flags Flags für die Darstellung
             */
            void sendPresented(uint32_t tvSecHi, uint32_t tvSecLo, uint32_t tvNsec, uint32_t refresh,
                               uint32_t sequenceHigh, uint32_t sequenceLow, uint32_t flags);

            /**
             * @brief Sendet eine Discarded-Rückmeldung
             */
            void sendDiscarded();

        signals:
            /**
             * @brief Signal, das ausgelöst wird, wenn das Feedback zerstört wird
             */
            void destroyed();

        private:
            QWaylandSurface* m_surface; ///< Wayland-Oberfläche
            QWaylandResource m_resource; ///< Wayland-Ressource
        };

        /**
         * @class PresentationTimeProtocol
         * @brief Implementierung des Presentation-Time-Protokolls
         *
         * Diese Klasse implementiert das Presentation-Time-Protokoll und ist für die
         * Verwaltung von Presentation-Feedback-Objekten verantwortlich.
         */
        class PresentationTimeProtocol : public QWaylandGlobal {
            Q_OBJECT

        public:
            /**
             * @brief Konstruktor
             * @param compositor Wayland-Compositor
             * @param parent Elternobjekt
             */
            explicit PresentationTimeProtocol(QWaylandCompositor* compositor, QObject* parent = nullptr);

            /**
             * @brief Destruktor
             */
            ~PresentationTimeProtocol();

            /**
             * @brief Erstellt ein neues Presentation-Feedback-Objekt
             * @param surface Wayland-Oberfläche
             * @param resource Wayland-Ressource
             * @return Neues Presentation-Feedback-Objekt
             */
            PresentationFeedback* createFeedback(QWaylandSurface* surface, QWaylandResource resource);

            /**
             * @brief Entfernt ein Presentation-Feedback-Objekt
             * @param feedback Presentation-Feedback-Objekt
             */
            void removeFeedback(PresentationFeedback* feedback);

            /**
             * @brief Gibt alle Presentation-Feedback-Objekte für eine Oberfläche zurück
             * @param surface Wayland-Oberfläche
             * @return Liste aller Presentation-Feedback-Objekte für die angegebene Oberfläche
             */
            QList<PresentationFeedback*> feedbacksForSurface(QWaylandSurface* surface) const;

            /**
             * @brief Benachrichtigt alle Feedback-Objekte für eine Oberfläche über eine erfolgreiche Darstellung
             * @param surface Wayland-Oberfläche, die dargestellt wurde
             * @param syncPoint Synchronisationspunkt für die Darstellung
             * @param frameTime Zeitpunkt der Darstellung in Nanosekunden. Wenn 0, wird der aktuelle Zeitpunkt verwendet.
             */
            void presentSurface(QWaylandSurface* surface, const QWaylandSyncPoint& syncPoint, uint64_t frameTime = 0);

            /**
             * @brief Benachrichtigt alle Feedback-Objekte für eine Oberfläche über eine verworfene Darstellung
             * @param surface Wayland-Oberfläche, deren Darstellung verworfen wurde
             */
            void discardSurface(QWaylandSurface* surface);

        signals:
            /**
             * @brief Signal, das ausgelöst wird, wenn ein neues Presentation-Feedback-Objekt erstellt wird
             * @param feedback Neues Presentation-Feedback-Objekt
             */
            void feedbackCreated(PresentationFeedback* feedback);

            /**
             * @brief Signal, das ausgelöst wird, wenn ein Presentation-Feedback-Objekt zerstört wird
             * @param feedback Zerstörtes Presentation-Feedback-Objekt
             */
            void feedbackDestroyed(PresentationFeedback* feedback);

        protected:
            /**
             * @brief Wird aufgerufen, wenn eine neue Ressource erstellt wird
             * @param client Wayland-Client
             * @param id Ressourcen-ID
             * @param version Protokollversion
             */
            void resourceCreated(QWaylandClient* client, uint id, int version) override;

        private:
            QMultiMap<QWaylandSurface*, PresentationFeedback*> m_feedbacks; ///< Zuordnung von Oberflächen zu Feedback-Objekten
        };

        /**
         * @brief Hilfsfunktion zum Aufteilen eines 64-Bit-Zeitwerts in zwei 32-Bit-Werte
         * @param time 64-Bit-Zeitwert
         * @param high Oberer 32-Bit-Wert (Ausgabe)
         * @param low Unterer 32-Bit-Wert (Ausgabe)
         */
        void splitTimeValue(uint64_t time, uint32_t& high, uint32_t& low);

    } // namespace Wayland
} // namespace VivoX
