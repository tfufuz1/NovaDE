/**
 * @file WaylandLayerShell.h
 * @brief Implementierung des wlr-layer-shell-Protokolls
 *
 * Diese Datei enthält die Implementierung des wlr-layer-shell-Protokolls, das für
 * die Erstellung von Oberflächen in verschiedenen Ebenen (Layers) verantwortlich ist.
 * Dies ist besonders wichtig für Panels, Docks und andere UI-Elemente, die über oder
 * unter normalen Fenstern angezeigt werden sollen.
 */

#pragma once

#include <QObject>
#include <QWaylandCompositor>
#include <QWaylandSurface>
#include <QWaylandResource>
#include <QWaylandGlobal>

namespace VivoX {
    namespace Wayland {

        /**
         * @brief Layer-Shell-Ebenen
         *
         * Diese Enum definiert die verschiedenen Ebenen, in denen Layer-Shell-Oberflächen
         * angezeigt werden können.
         */
        enum LayerShellLayer {
            BackgroundLayer = 0,  ///< Hintergrund-Ebene (unter allen anderen Ebenen)
            BottomLayer = 1,      ///< Untere Ebene (über Hintergrund, unter normalen Fenstern)
            TopLayer = 2,         ///< Obere Ebene (über normalen Fenstern, unter Overlay)
            OverlayLayer = 3      ///< Overlay-Ebene (über allen anderen Ebenen)
        };

        /**
         * @brief Layer-Shell-Anker
         *
         * Diese Flags definieren, an welchen Bildschirmrändern eine Layer-Shell-Oberfläche
         * verankert werden kann.
         */
        enum LayerShellAnchor {
            AnchorNone = 0,        ///< Keine Verankerung
            AnchorTop = (1 << 0),  ///< Verankerung am oberen Rand
            AnchorBottom = (1 << 1), ///< Verankerung am unteren Rand
            AnchorLeft = (1 << 2), ///< Verankerung am linken Rand
            AnchorRight = (1 << 3) ///< Verankerung am rechten Rand
        };

        /**
         * @brief Layer-Shell-Exklusivzone
         *
         * Diese Enum definiert, wie die exklusive Zone einer Layer-Shell-Oberfläche
         * behandelt werden soll.
         */
        enum LayerShellExclusiveZone {
            ExclusiveZoneNone = 0, ///< Keine exklusive Zone
            ExclusiveZoneAuto = -1 ///< Automatische exklusive Zone (basierend auf Größe)
        };

        // Vorwärtsdeklaration
        class WaylandLayerShell;

        /**
         * @class LayerSurface
         * @brief Repräsentiert eine Layer-Shell-Oberfläche
         *
         * Diese Klasse repräsentiert eine Layer-Shell-Oberfläche und enthält alle
         * Eigenschaften und Methoden, die für die Verwaltung einer solchen Oberfläche
         * benötigt werden.
         */
        class LayerSurface : public QObject {
            Q_OBJECT
            Q_PROPERTY(QWaylandSurface* surface READ surface CONSTANT)
            Q_PROPERTY(LayerShellLayer layer READ layer WRITE setLayer NOTIFY layerChanged)
            Q_PROPERTY(uint anchor READ anchor WRITE setAnchor NOTIFY anchorChanged)
            Q_PROPERTY(int exclusiveZone READ exclusiveZone WRITE setExclusiveZone NOTIFY exclusiveZoneChanged)
            Q_PROPERTY(QMargins margins READ margins WRITE setMargins NOTIFY marginsChanged)
            Q_PROPERTY(QSize desiredSize READ desiredSize WRITE setDesiredSize NOTIFY desiredSizeChanged)
            Q_PROPERTY(QString namespace READ getNamespace CONSTANT)

        public:
            /**
             * @brief Konstruktor
             * @param surface Wayland-Oberfläche
             * @param resource Wayland-Ressource
             * @param layer Ebene
             * @param namespace Namespace
             * @param parent Elternobjekt
             */
            explicit LayerSurface(QWaylandSurface* surface, QWaylandResource resource,
                                  LayerShellLayer layer, const QString& nameSpace, QObject* parent = nullptr);

            /**
             * @brief Destruktor
             */
            ~LayerSurface();

            /**
             * @brief Gibt die Wayland-Oberfläche zurück
             * @return Wayland-Oberfläche
             */
            QWaylandSurface* surface() const;

            /**
             * @brief Gibt die Ebene zurück
             * @return Ebene
             */
            LayerShellLayer layer() const;

            /**
             * @brief Setzt die Ebene
             * @param layer Ebene
             */
            void setLayer(LayerShellLayer layer);

            /**
             * @brief Gibt die Anker zurück
             * @return Anker
             */
            uint anchor() const;

            /**
             * @brief Setzt die Anker
             * @param anchor Anker
             */
            void setAnchor(uint anchor);

            /**
             * @brief Gibt die exklusive Zone zurück
             * @return Exklusive Zone
             */
            int exclusiveZone() const;

            /**
             * @brief Setzt die exklusive Zone
             * @param exclusiveZone Exklusive Zone
             */
            void setExclusiveZone(int exclusiveZone);

            /**
             * @brief Gibt die Ränder zurück
             * @return Ränder
             */
            QMargins margins() const;

            /**
             * @brief Setzt die Ränder
             * @param margins Ränder
             */
            void setMargins(const QMargins& margins);

            /**
             * @brief Gibt die gewünschte Größe zurück
             * @return Gewünschte Größe
             */
            QSize desiredSize() const;

            /**
             * @brief Setzt die gewünschte Größe
             * @param size Gewünschte Größe
             */
            void setDesiredSize(const QSize& size);

            /**
             * @brief Gibt den Namespace zurück
             * @return Namespace
             */
            QString getNamespace() const;

            /**
             * @brief Gibt die Wayland-Ressource zurück
             * @return Wayland-Ressource
             */
            QWaylandResource resource() const;

        signals:
            /**
             * @brief Signal, das ausgelöst wird, wenn sich die Ebene ändert
             * @param layer Neue Ebene
             */
            void layerChanged(LayerShellLayer layer);

            /**
             * @brief Signal, das ausgelöst wird, wenn sich die Anker ändern
             * @param anchor Neue Anker
             */
            void anchorChanged(uint anchor);

            /**
             * @brief Signal, das ausgelöst wird, wenn sich die exklusive Zone ändert
             * @param exclusiveZone Neue exklusive Zone
             */
            void exclusiveZoneChanged(int exclusiveZone);

            /**
             * @brief Signal, das ausgelöst wird, wenn sich die Ränder ändern
             * @param margins Neue Ränder
             */
            void marginsChanged(const QMargins& margins);

            /**
             * @brief Signal, das ausgelöst wird, wenn sich die gewünschte Größe ändert
             * @param size Neue gewünschte Größe
             */
            void desiredSizeChanged(const QSize& size);

            /**
             * @brief Signal, das ausgelöst wird, wenn die Oberfläche zerstört wird
             */
            void surfaceDestroyed();

        private:
            QWaylandSurface* m_surface; ///< Wayland-Oberfläche
            QWaylandResource m_resource; ///< Wayland-Ressource
            LayerShellLayer m_layer; ///< Ebene
            uint m_anchor; ///< Anker
            int m_exclusiveZone; ///< Exklusive Zone
            QMargins m_margins; ///< Ränder
            QSize m_desiredSize; ///< Gewünschte Größe
            QString m_namespace; ///< Namespace
        };

        /**
         * @class WaylandLayerShell
         * @brief Implementierung des wlr-layer-shell-Protokolls
         *
         * Diese Klasse implementiert das wlr-layer-shell-Protokoll und ist für die
         * Verwaltung von Layer-Shell-Oberflächen verantwortlich.
         */
        class WaylandLayerShell : public QWaylandGlobal {
            Q_OBJECT

        public:
            /**
             * @brief Konstruktor
             * @param compositor Wayland-Compositor
             * @param parent Elternobjekt
             */
            explicit WaylandLayerShell(QWaylandCompositor* compositor, QObject* parent = nullptr);

            /**
             * @brief Destruktor
             */
            ~WaylandLayerShell();

            /**
             * @brief Gibt alle Layer-Shell-Oberflächen zurück
             * @return Liste aller Layer-Shell-Oberflächen
             */
            QList<LayerSurface*> layerSurfaces() const;

            /**
             * @brief Gibt alle Layer-Shell-Oberflächen für eine bestimmte Ebene zurück
             * @param layer Ebene
             * @return Liste aller Layer-Shell-Oberflächen für die angegebene Ebene
             */
            QList<LayerSurface*> layerSurfacesForLayer(LayerShellLayer layer) const;

            /**
             * @brief Gibt alle Layer-Shell-Oberflächen für einen bestimmten Namespace zurück
             * @param nameSpace Namespace
             * @return Liste aller Layer-Shell-Oberflächen für den angegebenen Namespace
             */
            QList<LayerSurface*> layerSurfacesForNamespace(const QString& nameSpace) const;

            /**
             * @brief Findet eine Layer-Shell-Oberfläche für eine bestimmte Wayland-Oberfläche
             * @param surface Wayland-Oberfläche
             * @return Layer-Shell-Oberfläche oder nullptr, wenn keine gefunden wurde
             */
            LayerSurface* findLayerSurface(QWaylandSurface* surface) const;

            /**
             * @brief Konfiguriert eine Layer-Shell-Oberfläche
             * @param layerSurface Layer-Shell-Oberfläche
             * @param size Neue Größe
             */
            void configureSurface(LayerSurface* layerSurface, const QSize& size);

            /**
             * @brief Konfiguriert alle Layer-Shell-Oberflächen für eine bestimmte Ebene
             * @param layer Ebene
             * @param size Neue Größe
             */
            void configureSurfaces(LayerShellLayer layer, const QSize& size);

            /**
             * @brief Handler für das get_layer_surface-Protokoll-Request
             * @param surface Wayland-Oberfläche
             * @param resource Wayland-Ressource
             * @param layer Ebene
             * @param nameSpace Namespace
             */
            void handleGetLayerSurface(QWaylandSurface* surface, QWaylandResource resource,
                                       LayerShellLayer layer, const QString& nameSpace);

        signals:
            /**
             * @brief Signal, das ausgelöst wird, wenn eine neue Layer-Shell-Oberfläche erstellt wird
             * @param layerSurface Neue Layer-Shell-Oberfläche
             */
            void layerSurfaceCreated(LayerSurface* layerSurface);

            /**
             * @brief Signal, das ausgelöst wird, wenn eine Layer-Shell-Oberfläche zerstört wird
             * @param layerSurface Zerstörte Layer-Shell-Oberfläche
             */
            void layerSurfaceDestroyed(LayerSurface* layerSurface);

        protected:
            /**
             * @brief Wird aufgerufen, wenn eine neue Ressource erstellt wird
             * @param client Wayland-Client
             * @param id Ressourcen-ID
             * @param version Protokollversion
             */
            void resourceCreated(QWaylandClient* client, uint id, int version) override;

        private:
            QList<LayerSurface*> m_layerSurfaces; ///< Liste aller Layer-Shell-Oberflächen

            /**
             * @brief Erstellt eine neue Layer-Shell-Oberfläche
             * @param surface Wayland-Oberfläche
             * @param resource Wayland-Ressource
             * @param layer Ebene
             * @param namespace Namespace
             * @return Neue Layer-Shell-Oberfläche
             */
            LayerSurface* createLayerSurface(QWaylandSurface* surface, QWaylandResource resource,
                                             LayerShellLayer layer, const QString& nameSpace);

            /**
             * @brief Entfernt eine Layer-Shell-Oberfläche
             * @param layerSurface Layer-Shell-Oberfläche
             */
            void removeLayerSurface(LayerSurface* layerSurface);

            /**
             * @brief Sendet eine Konfiguration an eine Layer-Shell-Oberfläche
             * @param layerSurface Layer-Shell-Oberfläche
             * @param serial Seriennummer
             */
            void sendConfigure(LayerSurface* layerSurface, uint32_t serial);
        };

    } // namespace Wayland
} // namespace VivoX
