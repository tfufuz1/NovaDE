/**
 * @file ViewporterProtocol.h
 * @brief Implementierung des Viewporter-Protokolls
 *
 * Diese Datei enthält die Implementierung des Viewporter-Protokolls, das für
 * die Skalierung und Beschneidung von Oberflächen verantwortlich ist.
 */

#pragma once

#include <QObject>
#include <QWaylandCompositor>
#include <QWaylandSurface>
#include <QWaylandResource>
#include <QWaylandGlobal>
#include <QMap>
#include <QRectF>

namespace VivoX {
    namespace Wayland {

        // Forward-Deklaration für die private Klasse
        class ViewporterProtocolPrivate;

        /**
         * @class Viewport
         * @brief Repräsentiert ein Viewport-Objekt
         *
         * Diese Klasse repräsentiert ein Viewport-Objekt, das für die Skalierung und
         * Beschneidung einer Oberfläche verwendet wird.
         */
        class Viewport : public QObject {
            Q_OBJECT
            Q_PROPERTY(QWaylandSurface* surface READ surface CONSTANT)
            Q_PROPERTY(QRectF sourceRect READ sourceRect WRITE setSourceRect NOTIFY sourceRectChanged)
            Q_PROPERTY(QSizeF destinationSize READ destinationSize WRITE setDestinationSize NOTIFY destinationSizeChanged)

        public:
            /**
             * @brief Konstruktor
             * @param surface Wayland-Oberfläche
             * @param resource Wayland-Ressource
             * @param parent Elternobjekt
             */
            explicit Viewport(QWaylandSurface* surface, QWaylandResource resource, QObject* parent = nullptr);

            /**
             * @brief Destruktor
             */
            ~Viewport();

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
             * @brief Gibt das Quellrechteck zurück
             * @return Quellrechteck
             */
            QRectF sourceRect() const;

            /**
             * @brief Setzt das Quellrechteck
             * @param rect Quellrechteck
             */
            void setSourceRect(const QRectF& rect);

            /**
             * @brief Gibt die Zielgröße zurück
             * @return Zielgröße
             */
            QSizeF destinationSize() const;

            /**
             * @brief Setzt die Zielgröße
             * @param size Zielgröße
             */
            void setDestinationSize(const QSizeF& size);

            /**
             * @brief Prüft, ob das Viewport aktive Transformationen hat
             * @return true, wenn das Viewport aktive Transformationen hat
             */
            bool hasSourceRect() const { return !m_sourceRect.isEmpty(); }

            /**
             * @brief Prüft, ob das Viewport eine benutzerdefinierte Zielgröße hat
             * @return true, wenn das Viewport eine benutzerdefinierte Zielgröße hat
             */
            bool hasDestinationSize() const { return !m_destinationSize.isEmpty(); }

            /**
             * @brief Berechnet die effektive Größe der Oberfläche unter Berücksichtigung des Viewports
             * @param nativeSurfaceSize Natürliche Größe der Oberfläche
             * @return Effektive Größe der Oberfläche
             */
            QSizeF effectiveSize(const QSizeF& nativeSurfaceSize) const;

        signals:
            /**
             * @brief Signal, das ausgelöst wird, wenn sich das Quellrechteck ändert
             * @param rect Neues Quellrechteck
             */
            void sourceRectChanged(const QRectF& rect);

            /**
             * @brief Signal, das ausgelöst wird, wenn sich die Zielgröße ändert
             * @param size Neue Zielgröße
             */
            void destinationSizeChanged(const QSizeF& size);

            /**
             * @brief Signal, das ausgelöst wird, wenn das Viewport zerstört wird
             */
            void viewportDestroyed();

        private:
            QWaylandSurface* m_surface; ///< Wayland-Oberfläche
            QWaylandResource m_resource; ///< Wayland-Ressource
            QRectF m_sourceRect; ///< Quellrechteck
            QSizeF m_destinationSize; ///< Zielgröße
        };

        /**
         * @class ViewporterProtocol
         * @brief Implementierung des Viewporter-Protokolls
         *
         * Diese Klasse implementiert das Viewporter-Protokoll und ist für die
         * Verwaltung von Viewport-Objekten verantwortlich.
         */
        class ViewporterProtocol : public QWaylandGlobal {
            Q_OBJECT

        public:
            /**
             * @brief Konstruktor
             * @param compositor Wayland-Compositor
             * @param parent Elternobjekt
             */
            explicit ViewporterProtocol(QWaylandCompositor* compositor, QObject* parent = nullptr);

            /**
             * @brief Destruktor
             */
            ~ViewporterProtocol();

            /**
             * @brief Erstellt ein neues Viewport-Objekt
             * @param surface Wayland-Oberfläche
             * @param resource Wayland-Ressource
             * @return Neues Viewport-Objekt
             */
            Viewport* createViewport(QWaylandSurface* surface, QWaylandResource resource);

            /**
             * @brief Entfernt ein Viewport-Objekt
             * @param viewport Viewport-Objekt
             */
            void removeViewport(Viewport* viewport);

            /**
             * @brief Gibt das Viewport-Objekt für eine Oberfläche zurück
             * @param surface Wayland-Oberfläche
             * @return Viewport-Objekt für die angegebene Oberfläche
             */
            Viewport* viewportForSurface(QWaylandSurface* surface) const;

            /**
             * @brief Registriert das Viewporter-Protokoll beim Compositor
             * @param compositor Wayland-Compositor
             * @return Neue Instanz des Viewporter-Protokolls
             */
            static ViewporterProtocol* create(QWaylandCompositor* compositor);

            /**
             * @brief Prüft, ob eine Oberfläche ein aktives Viewport hat
             * @param surface Wayland-Oberfläche
             * @return true, wenn die Oberfläche ein aktives Viewport hat
             */
            bool hasViewport(QWaylandSurface* surface) const;

            /**
             * @brief Berechnet die effektive Größe einer Oberfläche unter Berücksichtigung des Viewports
             * @param surface Wayland-Oberfläche
             * @param nativeSurfaceSize Natürliche Größe der Oberfläche
             * @return Effektive Größe der Oberfläche
             */
            QSizeF effectiveSurfaceSize(QWaylandSurface* surface, const QSizeF& nativeSurfaceSize) const;

        signals:
            /**
             * @brief Signal, das ausgelöst wird, wenn ein neues Viewport-Objekt erstellt wird
             * @param viewport Neues Viewport-Objekt
             */
            void viewportCreated(Viewport* viewport);

            /**
             * @brief Signal, das ausgelöst wird, wenn ein Viewport-Objekt zerstört wird
             * @param viewport Zerstörtes Viewport-Objekt
             */
            void viewportDestroyed(Viewport* viewport);

        protected:
            /**
             * @brief Wird aufgerufen, wenn eine neue Ressource erstellt wird
             * @param client Wayland-Client
             * @param id Ressourcen-ID
             * @param version Protokollversion
             */
            void resourceCreated(QWaylandClient* client, uint id, int version) override;

            /**
             * @brief Freund-Deklaration für die private Klasse
             */
            friend class ViewporterProtocolPrivate;

        private:
            QMap<QWaylandSurface*, Viewport*> m_viewports; ///< Zuordnung von Oberflächen zu Viewport-Objekten
        };

    } // namespace Wayland
} // namespace VivoX
