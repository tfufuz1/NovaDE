/**
 * @file LinuxDmabufProtocol.h
 * @brief Definition des Linux-DMABUF-Protokolls
 *
 * Diese Datei enthält die Definition des Linux-DMABUF-Protokolls, das für
 * die effiziente Übertragung von Grafikpuffern zwischen Client und Compositor
 * verantwortlich ist.
 */

#ifndef VIVOX_WAYLAND_LINUXDMABUFPROTOCOL_H
#define VIVOX_WAYLAND_LINUXDMABUFPROTOCOL_H

#include <QObject>
#include <QVector>
#include <QWaylandCompositor>
#include <QWaylandGlobal>
#include <QWaylandResource>
#include <QMap>

#include <wayland-server.h>
#include <EGL/egl.h>
#include <EGL/eglext.h>

struct gbm_device;
struct gbm_bo;

// Vorwärtsdeklarationen
namespace VivoX {
    namespace Wayland {

        // Struktur für EGL-Funktionen
        struct EGLFunctions {
            // EGLImage-Erweiterungen
            PFNEGLCREATEIMAGEKHRPROC eglCreateImageKHR;
            PFNEGLDESTROYIMAGEKHRPROC eglDestroyImageKHR;

            // DMA-BUF-Erweiterungen
            PFNEGLQUERYDMABUFFORMATSEXTPROC eglQueryDmaBufFormatsEXT;
            PFNEGLQUERYDMABUFMODIFIERSEXTPROC eglQueryDmaBufModifiersEXT;
        };

        // Struktur zur Speicherung von Format- und Modifier-Informationen
        struct FormatModifier {
            uint32_t format;        // DRM-Format (z.B. DRM_FORMAT_ARGB8888)
            uint64_t modifier;      // DRM-Modifier (z.B. DRM_FORMAT_MOD_LINEAR)
            bool externalOnly;      // Nur für externe Texturen verfügbar

            bool operator==(const FormatModifier& other) const;
        };

        // Parameter für DMA-BUF-Buffer
        struct DmabufParams {
            int width;                  // Breite des Puffers in Pixeln
            int height;                 // Höhe des Puffers in Pixeln
            uint32_t format;            // DRM-Format (z.B. DRM_FORMAT_ARGB8888)
            uint32_t flags;             // Flags für den Puffer
            QVector<int> fds;           // Dateideskriptoren für jede Ebene
            QVector<uint32_t> offsets;  // Offsets für jede Ebene
            QVector<uint32_t> strides;  // Strides (Zeilenlängen) für jede Ebene
            QVector<uint64_t> modifiers; // Modifier für jede Ebene

            bool isValid() const;       // Überprüft, ob die Parameter gültig sind
        };

        /**
         * @brief Builder-Klasse für DmabufParams
         *
         * Diese Klasse wird verwendet, um DmabufParams-Objekte zu erstellen,
         * die für die Erstellung von LinuxDmabufBuffer-Objekten benötigt werden.
         */
        class LinuxDmabufParamsBuilder {
        public:
            LinuxDmabufParamsBuilder();
            ~LinuxDmabufParamsBuilder();

            LinuxDmabufParamsBuilder& setSize(int width, int height);
            LinuxDmabufParamsBuilder& setFormat(uint32_t format);
            LinuxDmabufParamsBuilder& setFlags(uint32_t flags);
            LinuxDmabufParamsBuilder& addPlane(int fd, uint32_t offset, uint32_t stride, uint64_t modifier = 0);

            DmabufParams build();

        private:
            DmabufParams m_params;
        };

        /**
         * @brief Klasse für Linux-DMABUF-Buffer
         *
         * Diese Klasse repräsentiert einen Buffer, der über das Linux-DMABUF-Protokoll
         * importiert wurde. Sie verwaltet die Ressourcen und kann zur Darstellung
         * verwendet werden.
         */
        class LinuxDmabufBuffer : public QObject {
            Q_OBJECT

        public:
            LinuxDmabufBuffer(QWaylandResource resource, const DmabufParams& params, QObject* parent = nullptr);
            ~LinuxDmabufBuffer();

            QWaylandResource resource() const;
            const DmabufParams& params() const;

            bool importBuffer(gbm_device* gbmDevice);
            bool importDmaBuffer(int drmFd);

            EGLImage createEGLImage(EGLDisplay eglDisplay);
            GLuint createGLTexture(EGLDisplay eglDisplay);

        signals:
            void destroyed();

        private:
            void releaseResources();
            bool initEGLFunctions(EGLDisplay eglDisplay);

            QWaylandResource m_resource;
            DmabufParams m_params;

            EGLImage m_eglImage;
            EGLDisplay m_eglDisplay;
            EGLFunctions m_eglFunctions;

            gbm_bo* m_gbmBo;
            GLuint m_texture;

            QVector<uint32_t> m_drmHandles;
        };

        /**
         * @brief Hauptklasse für das Linux-DMABUF-Protokoll
         *
         * Diese Klasse implementiert das Linux-DMABUF-Protokoll für den Compositor
         * und verwaltet die Buffer, die über dieses Protokoll erstellt werden.
         */
        class LinuxDmabufProtocol : public QWaylandGlobal {
            Q_OBJECT

        public:
            LinuxDmabufProtocol(QWaylandCompositor* compositor, QObject* parent = nullptr);
            ~LinuxDmabufProtocol();

            bool initialize();

            void setEGLDisplay(EGLDisplay display);
            void setDrmDevice(int drmFd);

            bool isFormatSupported(uint32_t format) const;
            bool isFormatModifierSupported(uint32_t format, uint64_t modifier) const;

            void addSupportedFormat(uint32_t format);
            void addSupportedFormatModifier(uint32_t format, uint64_t modifier, bool externalOnly = false);

            const QVector<uint32_t>& supportedFormats() const;
            const QVector<FormatModifier>& supportedFormatModifiers() const;

            gbm_device* gbmDevice() const;
            int drmFd() const;
            EGLDisplay eglDisplay() const;
            const EGLFunctions& eglFunctions() const;

            LinuxDmabufBuffer* getBuffer(QWaylandResource resource) const;

        private:
            // Implementation der zwp_linux_dmabuf_v1-Schnittstelle
            static const struct zwp_linux_dmabuf_v1_interface zwp_linux_dmabuf_v1_interface_implementation;
            static const struct zwp_linux_buffer_params_v1_interface zwp_linux_buffer_params_v1_interface_implementation;

            // Listener für die Zerstörung der Resource
            static wl_listener resource_destroy_params_listener[1];

            void bind(QWaylandClient* client, uint32_t version, uint32_t id) override;
            void initSupportedFormats();
            bool initEGLFunctions();
            void queryEGLFormatsAndModifiers();

            void addBuffer(LinuxDmabufBuffer* buffer);
            void removeBuffer(LinuxDmabufBuffer* buffer);

            // Handler für Protokoll-Anfragen
            void handle_create_params(wl_client* client, wl_resource* resource, uint32_t params_id);
            void handle_create_buffer(wl_resource* params_resource, int32_t width, int32_t height, uint32_t format, uint32_t flags);
            void handle_create_immed_buffer(wl_resource* params_resource, uint32_t buffer_id, int32_t width, int32_t height, uint32_t format, uint32_t flags);

            QVector<LinuxDmabufBuffer*> m_buffers;
            QVector<uint32_t> m_supportedFormats;
            QVector<FormatModifier> m_supportedFormatModifiers;

            EGLDisplay m_eglDisplay;
            EGLFunctions m_eglFunctions;

            gbm_device* m_gbmDevice;
            int m_drmFd;
        };

    } // namespace Wayland
} // namespace VivoX

#endif // VIVOX_WAYLAND_LINUXDMABUFPROTOCOL_H
