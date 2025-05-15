/**
 * @file LinuxDmabufProtocol.cpp
 * @brief Implementierung des Linux-DMABUF-Protokolls
 * 
 * Diese Datei enthält die Implementierung des Linux-DMABUF-Protokolls, das für
 * die effiziente Übertragung von Grafikpuffern zwischen Client und Compositor
 * verantwortlich ist.
 */

 #include "LinuxDmabufProtocol.h"
 #include "../core/Logger.h"
 #include <wayland-server-protocol.h>
 #include <unistd.h>
 #include <linux/dma-buf.h>
 #include <sys/ioctl.h>
 #include <fcntl.h>
 #include <string.h>
 #include <drm_fourcc.h>
 #include <QScopedPointer>
 #include <QDebug>
 #include <errno.h>
 #include <xf86drm.h>
 #include <gbm.h>
 #include <EGL/egl.h>
 #include <EGL/eglext.h>
 #include <GLES2/gl2.h>
 #include <GLES2/gl2ext.h>
 
 // Protokoll-Spezifikation für das Linux DMABUF Protokoll
 #include "wayland-linux-dmabuf-server-protocol.h"
 
 namespace VivoX {
 namespace Wayland {
 
 // ===============================
 // LinuxDmabufBuffer Implementation
 // ===============================
 
 LinuxDmabufBuffer::LinuxDmabufBuffer(QWaylandResource resource, const DmabufParams& params, QObject* parent)
     : QObject(parent)
     , m_resource(resource)
     , m_params(params)
     , m_eglImage(EGL_NO_IMAGE)
     , m_gbmBo(nullptr)
     , m_texture(0)
 {
     Core::Logger::instance().debug(
         QString("LinuxDmabufBuffer erstellt: Format=0x%1, Größe=%2x%3")
             .arg(params.format, 0, 16)
             .arg(params.width)
             .arg(params.height),
         "Wayland"
     );
     
     // Verbinde die destroy-Funktion mit dem Wayland-Resource-Destroy-Event
     wl_resource_set_implementation(resource.resource(),
                                   nullptr,
                                   this,
                                   [](wl_resource *resource) {
                                       LinuxDmabufBuffer *buffer = static_cast<LinuxDmabufBuffer*>(wl_resource_get_user_data(resource));
                                       if (buffer) {
                                           emit buffer->destroyed();
                                       }
                                   });
 }
 
 LinuxDmabufBuffer::~LinuxDmabufBuffer()
 {
     Core::Logger::instance().debug(
         QString("LinuxDmabufBuffer zerstört: Format=0x%1, Größe=%2x%3")
             .arg(m_params.format, 0, 16)
             .arg(m_params.width)
             .arg(m_params.height),
         "Wayland"
     );
     
     releaseResources();
 }
 
 QWaylandResource LinuxDmabufBuffer::resource() const
 {
     return m_resource;
 }
 
 const DmabufParams& LinuxDmabufBuffer::params() const
 {
     return m_params;
 }
 
 void LinuxDmabufBuffer::releaseResources()
 {
     // EGL-Image freigeben, falls es existiert
     if (m_eglImage != EGL_NO_IMAGE && m_eglFunctions.eglDestroyImageKHR) {
         m_eglFunctions.eglDestroyImageKHR(m_eglDisplay, m_eglImage);
         m_eglImage = EGL_NO_IMAGE;
     }
     
     // OpenGL-Textur freigeben, falls sie existiert
     if (m_texture) {
         glDeleteTextures(1, &m_texture);
         m_texture = 0;
     }
     
     // GBM-Buffer-Objekt freigeben, falls es existiert
     if (m_gbmBo) {
         gbm_bo_destroy(m_gbmBo);
         m_gbmBo = nullptr;
     }
 
     // Schließe alle Dateideskriptoren
     for (int fd : m_params.fds) {
         if (fd >= 0) {
             close(fd);
         }
     }
     
     // DRM Handles freigeben
     if (!m_drmHandles.isEmpty()) {
         for (uint32_t handle : m_drmHandles) {
             // Achtung: hier sollte eigentlich drmCloseHandle verwendet werden,
             // aber das hängt vom spezifischen DRM-Device ab
             // Dies würde eine zusätzliche Variable für den DRM-FD erfordern
         }
         m_drmHandles.clear();
     }
 }
 
 bool LinuxDmabufBuffer::importBuffer(gbm_device* gbmDevice)
 {
     // Überprüfen, ob die Parameter gültig sind
     if (m_params.width <= 0 || m_params.height <= 0 || m_params.format == 0 || m_params.fds.isEmpty()) {
         Core::Logger::instance().error("Ungültige DMABUF-Parameter", "Wayland");
         return false;
     }
 
     // Falls bereits ein Buffer importiert wurde, diesen freigeben
     if (m_gbmBo) {
         gbm_bo_destroy(m_gbmBo);
         m_gbmBo = nullptr;
     }
     
     // Plane-Informationen vorbereiten
     struct gbm_import_fd_modifier_data import_data;
     memset(&import_data, 0, sizeof(import_data));
     
     import_data.width = m_params.width;
     import_data.height = m_params.height;
     import_data.format = m_params.format;
     import_data.num_fds = m_params.fds.size();
     
     // Modifier zuweisen (falls vorhanden)
     if (!m_params.modifiers.isEmpty()) {
         import_data.modifier = m_params.modifiers[0];
     } else {
         import_data.modifier = DRM_FORMAT_MOD_LINEAR;
     }
     
     // Plane-Daten füllen
     for (int i = 0; i < m_params.fds.size() && i < 4; i++) {
         import_data.fds[i] = m_params.fds[i];
         import_data.strides[i] = m_params.strides[i];
         import_data.offsets[i] = m_params.offsets[i];
     }
     
     // Buffer über GBM importieren
     m_gbmBo = gbm_bo_import(gbmDevice, GBM_BO_IMPORT_FD_MODIFIER, &import_data, GBM_BO_USE_RENDERING);
     
     if (!m_gbmBo) {
         Core::Logger::instance().error(
             QString("Fehler beim Import des DMABUF-Puffers über GBM: %1").arg(strerror(errno)),
             "Wayland"
         );
         return false;
     }
     
     Core::Logger::instance().debug(
         QString("DMABUF-Puffer erfolgreich über GBM importiert: Format=0x%1, Größe=%2x%3")
             .arg(m_params.format, 0, 16)
             .arg(m_params.width)
             .arg(m_params.height),
         "Wayland"
     );
     
     return true;
 }
 
 bool LinuxDmabufBuffer::importDmaBuffer(int drmFd)
 {
     // Überprüfen, ob die Parameter gültig sind
     if (m_params.width <= 0 || m_params.height <= 0 || m_params.format == 0 || m_params.fds.isEmpty()) {
         Core::Logger::instance().error("Ungültige DMABUF-Parameter", "Wayland");
         return false;
     }
 
     // DMA-BUF importing mit dem DRM-Subsystem
     // PRIME-Import für jeden FD durchführen
     for (int i = 0; i < m_params.fds.size(); i++) {
         int fd = m_params.fds[i];
         uint32_t handle;
         
         int ret = drmPrimeFDToHandle(drmFd, fd, &handle);
         if (ret != 0) {
             Core::Logger::instance().error(
                 QString("Fehler bei DRM PRIME Import: %1").arg(strerror(errno)),
                 "Wayland"
             );
             return false;
         }
         
         // Handle in einer Sammlung speichern, wenn nötig
         m_drmHandles.append(handle);
     }
     
     Core::Logger::instance().debug(
         QString("DMABUF-Puffer erfolgreich über DRM PRIME importiert: Format=0x%1, Größe=%2x%3")
             .arg(m_params.format, 0, 16)
             .arg(m_params.width)
             .arg(m_params.height),
         "Wayland"
     );
     
     return !m_drmHandles.isEmpty();
 }
 
 EGLImage LinuxDmabufBuffer::createEGLImage(EGLDisplay eglDisplay)
 {
     // Wenn bereits ein EGL-Image existiert, dieses zurückgeben
     if (m_eglImage != EGL_NO_IMAGE) {
         return m_eglImage;
     }
     
     // EGL-Display speichern
     m_eglDisplay = eglDisplay;
     
     // EGL-Funktionen initialisieren
     if (!initEGLFunctions(eglDisplay)) {
         Core::Logger::instance().error("EGL-Funktionen konnten nicht initialisiert werden", "Wayland");
         return EGL_NO_IMAGE;
     }
     
     // Überprüfen, ob die Parameter gültig sind
     if (m_params.width <= 0 || m_params.height <= 0 || m_params.format == 0 || m_params.fds.isEmpty()) {
         Core::Logger::instance().error("Ungültige DMABUF-Parameter für EGLImage-Erstellung", "Wayland");
         return EGL_NO_IMAGE;
     }
     
     // Attribute für EGLImage erstellen
     QVector<EGLAttrib> attribs;
     
     // Grundlegende Attribute
     attribs << EGL_WIDTH << m_params.width;
     attribs << EGL_HEIGHT << m_params.height;
     attribs << EGL_LINUX_DRM_FOURCC_EXT << m_params.format;
     
     // Plane-Attribute für jede Plane hinzufügen
     for (int i = 0; i < m_params.fds.size() && i < 4; i++) {
         attribs << (EGL_DMA_BUF_PLANE0_FD_EXT + i * 3) << m_params.fds[i];
         attribs << (EGL_DMA_BUF_PLANE0_OFFSET_EXT + i * 3) << m_params.offsets[i];
         attribs << (EGL_DMA_BUF_PLANE0_PITCH_EXT + i * 3) << m_params.strides[i];
         
         // Modifier hinzufügen, falls vorhanden
         if (!m_params.modifiers.isEmpty() && i < m_params.modifiers.size()) {
             uint64_t modifier = m_params.modifiers[i];
             attribs << (EGL_DMA_BUF_PLANE0_MODIFIER_LO_EXT + i * 2) << static_cast<EGLAttrib>(modifier & 0xFFFFFFFF);
             attribs << (EGL_DMA_BUF_PLANE0_MODIFIER_HI_EXT + i * 2) << static_cast<EGLAttrib>(modifier >> 32);
         }
     }
     
     // Abschluss des Arrays
     attribs << EGL_NONE;
     
     // EGLImage erstellen
     m_eglImage = m_eglFunctions.eglCreateImageKHR(
         eglDisplay,
         EGL_NO_CONTEXT,
         EGL_LINUX_DMA_BUF_EXT,
         nullptr,
         attribs.data()
     );
     
     if (m_eglImage == EGL_NO_IMAGE) {
         Core::Logger::instance().error(
             QString("Fehler bei der EGLImage-Erstellung: EGL-Fehler 0x%1").arg(eglGetError(), 0, 16),
             "Wayland"
         );
         return EGL_NO_IMAGE;
     }
     
     Core::Logger::instance().debug(
         QString("EGLImage erfolgreich erstellt: Format=0x%1, Größe=%2x%3")
             .arg(m_params.format, 0, 16)
             .arg(m_params.width)
             .arg(m_params.height),
         "Wayland"
     );
     
     return m_eglImage;
 }
 
 bool LinuxDmabufBuffer::initEGLFunctions(EGLDisplay eglDisplay)
 {
     // Prüfe, ob die EGL-Erweiterungen verfügbar sind
     const char* extensions = eglQueryString(eglDisplay, EGL_EXTENSIONS);
     if (!extensions) {
         Core::Logger::instance().error("Konnte EGL-Erweiterungen nicht abfragen", "Wayland");
         return false;
     }
     
     // Prüfen, ob die notwendigen Erweiterungen vorhanden sind
     bool hasImageBase = strstr(extensions, "EGL_KHR_image_base") != nullptr;
     bool hasDmaBufImport = strstr(extensions, "EGL_EXT_image_dma_buf_import") != nullptr;
     bool hasDmaBufImportModifiers = strstr(extensions, "EGL_EXT_image_dma_buf_import_modifiers") != nullptr;
     
     if (!hasImageBase || !hasDmaBufImport) {
         Core::Logger::instance().error(
             "Benötigte EGL-Erweiterungen nicht vorhanden: EGL_KHR_image_base, EGL_EXT_image_dma_buf_import",
             "Wayland"
         );
         return false;
     }
     
     // Notwendige EGL-Funktionen laden
     m_eglFunctions.eglCreateImageKHR = reinterpret_cast<PFNEGLCREATEIMAGEKHRPROC>(
         eglGetProcAddress("eglCreateImageKHR"));
     m_eglFunctions.eglDestroyImageKHR = reinterpret_cast<PFNEGLDESTROYIMAGEKHRPROC>(
         eglGetProcAddress("eglDestroyImageKHR"));
     
     if (!m_eglFunctions.eglCreateImageKHR || !m_eglFunctions.eglDestroyImageKHR) {
         Core::Logger::instance().error("EGL-Funktionen konnten nicht geladen werden", "Wayland");
         return false;
     }
     
     // Lade optionale Funktionen für Modifier-Support
     if (hasDmaBufImportModifiers) {
         m_eglFunctions.eglQueryDmaBufFormatsEXT = reinterpret_cast<PFNEGLQUERYDMABUFFORMATSEXTPROC>(
             eglGetProcAddress("eglQueryDmaBufFormatsEXT"));
         m_eglFunctions.eglQueryDmaBufModifiersEXT = reinterpret_cast<PFNEGLQUERYDMABUFMODIFIERSEXTPROC>(
             eglGetProcAddress("eglQueryDmaBufModifiersEXT"));
     }
     
     return true;
 }
 
 GLuint LinuxDmabufBuffer::createGLTexture(EGLDisplay eglDisplay)
 {
     // Wenn bereits eine Textur existiert, diese zurückgeben
     if (m_texture != 0) {
         return m_texture;
     }
     
     // Zuerst ein EGLImage erstellen
     EGLImage image = createEGLImage(eglDisplay);
     if (image == EGL_NO_IMAGE) {
         return 0;
     }
     
     // Zugriff auf die EGL-GL-Schnittstellen
     PFNGLEGLIMAGETARGETTEXTURE2DOESPROC glEGLImageTargetTexture2DOES = 
         reinterpret_cast<PFNGLEGLIMAGETARGETTEXTURE2DOESPROC>(eglGetProcAddress("glEGLImageTargetTexture2DOES"));
     
     if (!glEGLImageTargetTexture2DOES) {
         Core::Logger::instance().error("glEGLImageTargetTexture2DOES nicht verfügbar", "Wayland");
         return 0;
     }
     
     // Textur erstellen und binden
     GLuint texture;
     glGenTextures(1, &texture);
     glBindTexture(GL_TEXTURE_2D, texture);
     
     // EGLImage auf Textur anwenden
     glEGLImageTargetTexture2DOES(GL_TEXTURE_2D, image);
     
     // Textur-Parameter setzen
     glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
     glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
     glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
     glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
     
     // Textur speichern
     m_texture = texture;
     
     Core::Logger::instance().debug(
         QString("GL-Textur erfolgreich erstellt: ID=%1").arg(texture),
         "Wayland"
     );
     
     return texture;
 }
 
 // ===============================
 // DmabufParams Implementation
 // ===============================
 
 bool DmabufParams::isValid() const
 {
     return width > 0 && height > 0 && format != 0 && !fds.isEmpty() &&
            fds.size() == offsets.size() && fds.size() == strides.size() &&
            (modifiers.isEmpty() || fds.size() == modifiers.size());
 }
 
 // ===============================
 // LinuxDmabufParamsBuilder Implementation
 // ===============================
 
 LinuxDmabufParamsBuilder::LinuxDmabufParamsBuilder()
     : m_params()
 {
     m_params.width = 0;
     m_params.height = 0;
     m_params.format = 0;
     m_params.flags = 0;
 }
 
 LinuxDmabufParamsBuilder::~LinuxDmabufParamsBuilder()
 {
     // Schließe alle FDs, die noch nicht an die endgültigen Parameter übergeben wurden
     for (int fd : m_params.fds) {
         if (fd >= 0) {
             close(fd);
         }
     }
 }
 
 LinuxDmabufParamsBuilder& LinuxDmabufParamsBuilder::setSize(int width, int height)
 {
     m_params.width = width;
     m_params.height = height;
     return *this;
 }
 
 LinuxDmabufParamsBuilder& LinuxDmabufParamsBuilder::setFormat(uint32_t format)
 {
     m_params.format = format;
     return *this;
 }
 
 LinuxDmabufParamsBuilder& LinuxDmabufParamsBuilder::setFlags(uint32_t flags)
 {
     m_params.flags = flags;
     return *this;
 }
 
 LinuxDmabufParamsBuilder& LinuxDmabufParamsBuilder::addPlane(int fd, uint32_t offset, uint32_t stride, uint64_t modifier)
 {
     // Dupliziere den FD, da wir die Kontrolle über das Schließen behalten wollen
     int dupFd = ::dup(fd);
     if (dupFd < 0) {
         Core::Logger::instance().error(
             QString("Fehler beim Duplizieren des FD: %1").arg(strerror(errno)),
             "Wayland"
         );
         return *this;
     }
     
     m_params.fds.append(dupFd);
     m_params.offsets.append(offset);
     m_params.strides.append(stride);
     
     // Modifier hinzufügen, wenn != DRM_FORMAT_MOD_INVALID
     if (modifier != DRM_FORMAT_MOD_INVALID) {
         // Stelle sicher, dass für alle Planes Modifier gesetzt sind
         while (m_params.modifiers.size() < m_params.fds.size() - 1) {
             m_params.modifiers.append(DRM_FORMAT_MOD_INVALID);
         }
         m_params.modifiers.append(modifier);
     }
     
     return *this;
 }
 
 DmabufParams LinuxDmabufParamsBuilder::build()
 {
     DmabufParams params = m_params;
     
     // Leere die internen Parameter, um zu verhindern, dass die FDs geschlossen werden
     m_params.fds.clear();
     m_params.offsets.clear();
     m_params.strides.clear();
     m_params.modifiers.clear();
     
     return params;
 }
 
 // ===============================
 // FormatModifier Implementation
 // ===============================
 
 bool FormatModifier::operator==(const FormatModifier& other) const
 {
     return format == other.format && modifier == other.modifier;
 }
 
 // ===============================
 // LinuxDmabufProtocol Implementation
 // ===============================
 
 LinuxDmabufProtocol::LinuxDmabufProtocol(QWaylandCompositor* compositor, QObject* parent)
     : QWaylandGlobal(compositor, 3, "zwp_linux_dmabuf_v1", parent)
     , m_buffers()
     , m_supportedFormats()
     , m_supportedFormatModifiers()
     , m_eglDisplay(EGL_NO_DISPLAY)
     , m_gbmDevice(nullptr)
     , m_drmFd(-1)
 {
     Core::Logger::instance().info("LinuxDmabufProtocol erstellt", "Wayland");
 }
 
 LinuxDmabufProtocol::~LinuxDmabufProtocol()
 {
     // Entferne alle Puffer
     while (!m_buffers.isEmpty()) {
         removeBuffer(m_buffers.first());
     }
     
     // Gib GBM-Device frei, falls es existiert
     if (m_gbmDevice) {
         gbm_device_destroy(m_gbmDevice);
         m_gbmDevice = nullptr;
     }
     
     // Schließe DRM-FD, falls er geöffnet ist
     if (m_drmFd >= 0) {
         close(m_drmFd);
         m_drmFd = -1;
     }
     
     Core::Logger::instance().info("LinuxDmabufProtocol zerstört", "Wayland");
 }
 
 bool LinuxDmabufProtocol::initialize()
 {
     // Initialisiere EGL-Funktionen, falls EGL-Display bereits gesetzt ist
     if (m_eglDisplay != EGL_NO_DISPLAY) {
         if (!initEGLFunctions()) {
             Core::Logger::instance().warning("EGL-Funktionen konnten nicht initialisiert werden", "Wayland");
         }
     }
     
     // Initialisiere die Liste der unterstützten Formate
     initSupportedFormats();
     
     return true;
 }
 
 void LinuxDmabufProtocol::setEGLDisplay(EGLDisplay display)
 {
     m_eglDisplay = display;
     
     // Wenn bereits ein EGL-Display gesetzt wird, versuche die EGL-Funktionen zu initialisieren
     if (m_eglDisplay != EGL_NO_DISPLAY) {
         if (initEGLFunctions()) {
             // Wenn die EGL-Funktionen erfolgreich initialisiert wurden, aktualisiere die Formate
             queryEGLFormatsAndModifiers();
         }
     }
 }
 
 void LinuxDmabufProtocol::setDrmDevice(int drmFd)
 {
     // Falls bereits ein DRM-FD gesetzt ist, diesen schließen
     if (m_drmFd >= 0) {
         close(m_drmFd);
     }
     
     // DRM-FD speichern
     m_drmFd = ::dup(drmFd);
     
     // Falls bereits ein GBM-Device existiert, dieses freigeben
     if (m_gbmDevice) {
         gbm_device_destroy(m_gbmDevice);
         m_gbmDevice = nullptr;
     }
     
     // Neues GBM-Device erstellen
     m_gbmDevice = gbm_create_device(m_drmFd);
     if (!m_gbmDevice) {
         Core::Logger::instance().error(
             QString("Fehler beim Erstellen des GBM-Devices: %1").arg(strerror(errno)),
             "Wayland"
         );
     }
 }
 
 bool LinuxDmabufProtocol::initEGLFunctions()
 {
     if (m_eglDisplay == EGL_NO_DISPLAY) {
         Core::Logger::instance().warning("Kein EGL-Display gesetzt, EGL-Funktionen können nicht initialisiert werden", "Wayland");
         return false;
     }
     
     // Prüfe, ob die EGL-Erweiterungen verfügbar sind
     const char* extensions = eglQueryString(m_eglDisplay, EGL_EXTENSIONS);
     if (!extensions) {
         Core::Logger::instance().error("Konnte EGL-Erweiterungen nicht abfragen", "Wayland");
         return false;
     }
     
     // Prüfen, ob die notwendigen Erweiterungen vorhanden sind
     bool hasImageBase = strstr(extensions, "EGL_KHR_image_base") != nullptr;
     bool hasDmaBufImport = strstr(extensions, "EGL_EXT_image_dma_buf_import") != nullptr;
     bool hasDmaBufImportModifiers = strstr(extensions, "EGL_EXT_image_dma_buf_import_modifiers") != nullptr;
     
     if (!hasImageBase || !hasDmaBufImport) {
         Core::Logger::instance().error(
             "Benötigte EGL-Erweiterungen nicht vorhanden: EGL_KHR_image_base, EGL_EXT_image_dma_buf_import",
             "Wayland"
         );
         return false;
     }
     
     // Notwendige EGL-Funktionen laden
     m_eglFunctions.eglCreateImageKHR = reinterpret_cast<PFNEGLCREATEIMAGEKHRPROC>(
         eglGetProcAddress("eglCreateImageKHR"));
     m_eglFunctions.eglDestroyImageKHR = reinterpret_cast<PFNEGLDESTROYIMAGEKHRPROC>(
         eglGetProcAddress("eglDestroyImageKHR"));
     
     if (!m_eglFunctions.eglCreateImageKHR || !m_eglFunctions.eglDestroyImageKHR) {
         Core::Logger::instance().error("EGL-Funktionen konnten nicht geladen werden", "Wayland");
         return false;
     }
     
     // Optionale Funktionen für Modifier-Support laden
     if (hasDmaBufImportModifiers) {
         m_eglFunctions.eglQueryDmaBufFormatsEXT = reinterpret_cast<PFNEGLQUERYDMABUFFORMATSEXTPROC>(
             eglGetProcAddress("eglQueryDmaBufFormatsEXT"));
         m_eglFunctions.eglQueryDmaBufModifiersEXT = reinterpret_cast<PFNEGLQUERYDMABUFMODIFIERSEXTPROC>(
             eglGetProcAddress("eglQueryDmaBufModifiersEXT"));
         
         // Wenn Modifier-Support vorhanden ist, frage die unterstützten Formate und Modifier ab
         if (m_eglFunctions.eglQueryDmaBufFormatsEXT && m_eglFunctions.eglQueryDmaBufModifiersEXT) {
             Core::Logger::instance().info("Abfrage unterstützter Formate und Modifier über EGL", "Wayland");
             queryEGLFormatsAndModifiers();
         }
     }
     
     Core::Logger::instance().info("EGL-Funktionen erfolgreich initialisiert", "Wayland");
     return true;
 }
 
 void LinuxDmabufProtocol::queryEGLFormatsAndModifiers()
 {
     // Abfrage der verfügbaren DMA-BUF-Formate
     EGLint formatCount = 0;
     if (!m_eglFunctions.eglQueryDmaBufFormatsEXT(m_eglDisplay, 0, nullptr, &formatCount) || formatCount == 0) {
         Core::Logger::instance().warning("Keine DMA-BUF-Formate verfügbar", "Wayland");
         return;
     }
     
     // Formate abfragen
     QVector<EGLint> formats(formatCount);
     if (!m_eglFunctions.eglQueryDmaBufFormatsEXT(m_eglDisplay, formatCount, formats.data(), &formatCount)) {
         Core::Logger::instance().warning("Fehler bei der Abfrage der DMA-BUF-Formate", "Wayland");
         return;
     }
     
     // Leere zuerst die Listen, um sie neu zu befüllen
     m_supportedFormats.clear();
     m_supportedFormatModifiers.clear();
     
     // Formate zu m_supportedFormats hinzufügen
     for (EGLint format : formats) {
         uint32_t drmFormat = static_cast<uint32_t>(format);
         if (!m_supportedFormats.contains(drmFormat)) {
             m_supportedFormats.append(drmFormat);
         }
         
// Für jedes Format die Modifier abfragen
EGLint modifierCount = 0;
if (m_eglFunctions.eglQueryDmaBufModifiersEXT(m_eglDisplay, format, 0, nullptr, nullptr, &modifierCount)) {
    if (modifierCount > 0) {
        // Arrays für Modifier und externe Flags allokieren
        QVector<uint64_t> modifiers(modifierCount);
        QVector<EGLBoolean> externalOnly(modifierCount);
        
        // Modifier für das aktuelle Format abfragen
        if (m_eglFunctions.eglQueryDmaBufModifiersEXT(
                m_eglDisplay, format, modifierCount,
                modifiers.data(), externalOnly.data(), &modifierCount)) {
            
            // Die Modifier für das aktuelle Format zur Liste hinzufügen
            for (int i = 0; i < modifierCount; ++i) {
                FormatModifier formatModifier;
                formatModifier.format = drmFormat;
                formatModifier.modifier = modifiers[i];
                formatModifier.externalOnly = externalOnly[i] == EGL_TRUE;
                
                if (!m_supportedFormatModifiers.contains(formatModifier)) {
                    m_supportedFormatModifiers.append(formatModifier);
                    
                    Core::Logger::instance().debug(
                        QString("Unterstütztes Format und Modifier: Format=0x%1, Modifier=0x%2%3")
                            .arg(formatModifier.format, 0, 16)
                            .arg(formatModifier.modifier >> 32, 0, 16)
                            .arg(formatModifier.modifier & 0xFFFFFFFF, 8, 16, QChar('0')),
                        "Wayland"
                    );
                }
            }
        }
    } else {
        // Wenn keine Modifier für das Format verfügbar sind, füge es mit dem impliziten Modifier hinzu
        FormatModifier formatModifier;
        formatModifier.format = drmFormat;
        formatModifier.modifier = DRM_FORMAT_MOD_INVALID;
        formatModifier.externalOnly = false;
        
        if (!m_supportedFormatModifiers.contains(formatModifier)) {
            m_supportedFormatModifiers.append(formatModifier);
        }
    }
}
}

Core::Logger::instance().info(
QString("EGL DMA-BUF: %1 Formate und %2 Format-Modifier-Kombinationen gefunden")
    .arg(m_supportedFormats.size())
    .arg(m_supportedFormatModifiers.size()),
"Wayland"
);
}

void LinuxDmabufProtocol::initSupportedFormats()
{
// Standardmäßig werden einige häufig verwendete Formate hinzugefügt
// Dies ist eine Fallback-Liste für den Fall, dass die EGL-Abfrage fehlschlägt

if (m_supportedFormats.isEmpty()) {
// RGB-Formate
m_supportedFormats.append(DRM_FORMAT_XRGB8888);
m_supportedFormats.append(DRM_FORMAT_ARGB8888);
m_supportedFormats.append(DRM_FORMAT_XBGR8888);
m_supportedFormats.append(DRM_FORMAT_ABGR8888);

// YUV-Formate
m_supportedFormats.append(DRM_FORMAT_NV12);
m_supportedFormats.append(DRM_FORMAT_YUV420);
m_supportedFormats.append(DRM_FORMAT_YUYV);

Core::Logger::instance().info(
    QString("Standard-DMA-BUF-Formate initialisiert: %1 Formate")
        .arg(m_supportedFormats.size()),
    "Wayland"
);
}

// Füge für jedes Format einen Standardmodifier hinzu, wenn keine Modifier abgefragt wurden
if (m_supportedFormatModifiers.isEmpty()) {
for (uint32_t format : m_supportedFormats) {
    // Linear-Modifier für jedes Format hinzufügen
    FormatModifier formatModifier;
    formatModifier.format = format;
    formatModifier.modifier = DRM_FORMAT_MOD_LINEAR;
    formatModifier.externalOnly = false;
    
    m_supportedFormatModifiers.append(formatModifier);
}

Core::Logger::instance().info(
    QString("Standard-Format-Modifier initialisiert: %1 Kombinationen")
        .arg(m_supportedFormatModifiers.size()),
    "Wayland"
);
}
}

bool LinuxDmabufProtocol::isFormatSupported(uint32_t format) const
{
return m_supportedFormats.contains(format);
}

bool LinuxDmabufProtocol::isFormatModifierSupported(uint32_t format, uint64_t modifier) const
{
FormatModifier formatModifier;
formatModifier.format = format;
formatModifier.modifier = modifier;

return m_supportedFormatModifiers.contains(formatModifier);
}

void LinuxDmabufProtocol::addSupportedFormat(uint32_t format)
{
if (!m_supportedFormats.contains(format)) {
m_supportedFormats.append(format);

// Standardmäßig auch den LINEAR-Modifier hinzufügen
FormatModifier formatModifier;
formatModifier.format = format;
formatModifier.modifier = DRM_FORMAT_MOD_LINEAR;
formatModifier.externalOnly = false;

if (!m_supportedFormatModifiers.contains(formatModifier)) {
    m_supportedFormatModifiers.append(formatModifier);
}
}
}

void LinuxDmabufProtocol::addSupportedFormatModifier(uint32_t format, uint64_t modifier, bool externalOnly)
{
// Stelle sicher, dass das Format in der Liste ist
if (!m_supportedFormats.contains(format)) {
m_supportedFormats.append(format);
}

// Stelle sicher, dass die Format-Modifier-Kombination in der Liste ist
FormatModifier formatModifier;
formatModifier.format = format;
formatModifier.modifier = modifier;
formatModifier.externalOnly = externalOnly;

if (!m_supportedFormatModifiers.contains(formatModifier)) {
m_supportedFormatModifiers.append(formatModifier);
}
}

LinuxDmabufBuffer* LinuxDmabufProtocol::getBuffer(QWaylandResource resource) const
{
for (LinuxDmabufBuffer* buffer : m_buffers) {
if (buffer->resource() == resource) {
    return buffer;
}
}

return nullptr;
}

void LinuxDmabufProtocol::addBuffer(LinuxDmabufBuffer* buffer)
{
if (!m_buffers.contains(buffer)) {
m_buffers.append(buffer);

// Verbinde das destroyed-Signal des Puffers mit unserem Slot
connect(buffer, &LinuxDmabufBuffer::destroyed, this, [this, buffer]() {
    removeBuffer(buffer);
});
}
}

void LinuxDmabufProtocol::removeBuffer(LinuxDmabufBuffer* buffer)
{
if (m_buffers.contains(buffer)) {
m_buffers.removeOne(buffer);

// Wenn der Buffer noch nicht gelöscht wurde, machen wir es jetzt
buffer->deleteLater();
}
}

void LinuxDmabufProtocol::bind(QWaylandClient* client, uint32_t version, uint32_t id)
{
QWaylandResource resource = client->createResource(interfaceName(), version, id);

// Protokoll-Implementierung an die Resource anhängen
wl_resource_set_implementation(
resource.resource(),
&zwp_linux_dmabuf_v1_interface_implementation,
this,
nullptr
);

// Unterstützte Formate an den Client senden
if (version >= ZWP_LINUX_DMABUF_V1_GET_DEFAULT_FEEDBACK_SINCE_VERSION) {
// Ab Version 4 des Protokolls verwenden wir das Feedback-Objekt
// In dieser Implementierung unterstützen wir derzeit nur bis Version 3
Core::Logger::instance().debug(
    "Client verwendet v4+ des Linux-DMABUF-Protokolls, aber wir implementieren nur bis v3",
    "Wayland"
);
} else {
// Für Version 3 senden wir die Formate direkt
for (uint32_t format : m_supportedFormats) {
    // In Version 3 senden wir die Formate einzeln
    if (version >= ZWP_LINUX_DMABUF_V1_MODIFIER_SINCE_VERSION) {
        // Mit Modifiern
        for (const FormatModifier& formatModifier : m_supportedFormatModifiers) {
            if (formatModifier.format == format) {
                uint32_t highBits = static_cast<uint32_t>(formatModifier.modifier >> 32);
                uint32_t lowBits = static_cast<uint32_t>(formatModifier.modifier & 0xFFFFFFFF);
                
                zwp_linux_dmabuf_v1_send_modifier(
                    resource.resource(),
                    format,
                    highBits,
                    lowBits
                );
            }
        }
    } else {
        // Ohne Modifier (für Version < 3)
        zwp_linux_dmabuf_v1_send_format(resource.resource(), format);
    }
}
}
}

// Statische Implementierung der Wayland-Protokoll-Funktionen

const struct zwp_linux_dmabuf_v1_interface LinuxDmabufProtocol::zwp_linux_dmabuf_v1_interface_implementation = {
// create_params
[](wl_client* client, wl_resource* resource, uint32_t params_id) {
LinuxDmabufProtocol* self = static_cast<LinuxDmabufProtocol*>(wl_resource_get_user_data(resource));
self->handle_create_params(client, resource, params_id);
},
// destroy
[](wl_client* client, wl_resource* resource) {
Q_UNUSED(client);
wl_resource_destroy(resource);
},
// get_default_feedback (ab Version 4, nicht implementiert)
nullptr,
// get_surface_feedback (ab Version 4, nicht implementiert)
nullptr
};

const struct zwp_linux_buffer_params_v1_interface LinuxDmabufProtocol::zwp_linux_buffer_params_v1_interface_implementation = {
// destroy
[](wl_client* client, wl_resource* resource) {
Q_UNUSED(client);
wl_resource_destroy(resource);
},
// add
[](wl_client* client, wl_resource* resource, int32_t fd, uint32_t plane_idx, uint32_t offset, uint32_t stride, uint32_t modifier_hi, uint32_t modifier_lo) {
Q_UNUSED(client);
LinuxDmabufParamsBuilder* builder = static_cast<LinuxDmabufParamsBuilder*>(wl_resource_get_user_data(resource));
uint64_t modifier = ((uint64_t)modifier_hi << 32) | modifier_lo;
builder->addPlane(fd, offset, stride, modifier);
},
// create
[](wl_client* client, wl_resource* resource, int32_t width, int32_t height, uint32_t format, uint32_t flags) {
Q_UNUSED(client);
LinuxDmabufProtocol* self = static_cast<LinuxDmabufProtocol*>(wl_resource_get_destroy_listener(resource, LinuxDmabufProtocol::resource_destroy_params_listener)[0].data);
self->handle_create_buffer(resource, width, height, format, flags);
},
// create_immed (ab Version 2)
[](wl_client* client, wl_resource* resource, uint32_t buffer_id, int32_t width, int32_t height, uint32_t format, uint32_t flags) {
Q_UNUSED(client);
LinuxDmabufProtocol* self = static_cast<LinuxDmabufProtocol*>(wl_resource_get_destroy_listener(resource, LinuxDmabufProtocol::resource_destroy_params_listener)[0].data);
self->handle_create_immed_buffer(resource, buffer_id, width, height, format, flags);
}
};

void LinuxDmabufProtocol::handle_create_params(wl_client* client, wl_resource* resource, uint32_t params_id)
{
Q_UNUSED(resource);

uint32_t version = wl_resource_get_version(resource);
QWaylandResource paramsResource = QWaylandResource::fromClient(client, zwp_linux_buffer_params_v1_interface, version, params_id);

// Parameter-Builder erstellen
LinuxDmabufParamsBuilder* builder = new LinuxDmabufParamsBuilder();

// Implementation des Wayland-Objekts setzen
wl_resource_set_implementation(
paramsResource.resource(),
&zwp_linux_buffer_params_v1_interface_implementation,
builder,
[](wl_resource* resource) {
    // Aufräumen beim Zerstören der Resource
    LinuxDmabufParamsBuilder* builder = static_cast<LinuxDmabufParamsBuilder*>(wl_resource_get_user_data(resource));
    delete builder;
}
);

// Destroy-Listener für uns selbst hinzufügen
wl_resource_add_destroy_listener(
paramsResource.resource(),
resource_destroy_params_listener
);

// Speichere den Protokoll-Handler als destroy_listener-Daten
resource_destroy_params_listener[0].data = this;
}

void LinuxDmabufProtocol::handle_create_buffer(wl_resource* params_resource, int32_t width, int32_t height, uint32_t format, uint32_t flags)
{
// Builder abrufen
LinuxDmabufParamsBuilder* builder = static_cast<LinuxDmabufParamsBuilder*>(wl_resource_get_user_data(params_resource));

// Parameter für den Buffer erstellen
builder->setSize(width, height)
   .setFormat(format)
   .setFlags(flags);

DmabufParams params = builder->build();

// Überprüfe, ob die Parameter gültig sind
if (!params.isValid()) {
zwp_linux_buffer_params_v1_send_failed(params_resource);
return;
}

// Überprüfe, ob das Format unterstützt wird
if (!isFormatSupported(params.format)) {
zwp_linux_buffer_params_v1_send_failed(params_resource);
Core::Logger::instance().warning(
    QString("Nicht unterstütztes Format: 0x%1").arg(params.format, 0, 16),
    "Wayland"
);
return;
}

// Überprüfe, ob die Format-Modifier-Kombination unterstützt wird
if (!params.modifiers.isEmpty()) {
bool modifierSupported = false;

for (uint64_t modifier : params.modifiers) {
    if (isFormatModifierSupported(params.format, modifier)) {
        modifierSupported = true;
        break;
    }
}

if (!modifierSupported) {
    zwp_linux_buffer_params_v1_send_failed(params_resource);
    Core::Logger::instance().warning(
        QString("Nicht unterstützte Format-Modifier-Kombination: Format=0x%1, Modifier=0x%2")
            .arg(params.format, 0, 16)
            .arg(params.modifiers[0], 0, 16),
        "Wayland"
    );
    return;
}
}

// Erstelle ein neues buffer Objekt für den Client
int version = wl_resource_get_version(params_resource);
wl_client* client = wl_resource_get_client(params_resource);
QWaylandResource bufferResource = QWaylandResource::fromClient(client, wl_buffer_interface, 1, 0);

// Erstelle einen LinuxDmabufBuffer
LinuxDmabufBuffer* buffer = new LinuxDmabufBuffer(bufferResource, params, this);

// Füge den Puffer zur Liste hinzu
addBuffer(buffer);

// Sende die Buffer-ID an den Client
zwp_linux_buffer_params_v1_send_created(params_resource, bufferResource.resource());
}

void LinuxDmabufProtocol::handle_create_immed_buffer(wl_resource* params_resource, uint32_t buffer_id, int32_t width, int32_t height, uint32_t format, uint32_t flags)
{
// Builder abrufen
LinuxDmabufParamsBuilder* builder = static_cast<LinuxDmabufParamsBuilder*>(wl_resource_get_user_data(params_resource));

// Parameter für den Buffer erstellen
builder->setSize(width, height)
   .setFormat(format)
   .setFlags(flags);

DmabufParams params = builder->build();

// Überprüfe, ob die Parameter gültig sind
if (!params.isValid()) {
// In diesem Fall können wir keinen Fehler zurückgeben, da dies ein sofortiger Buffer ist
// Wir erstellen einen leeren Buffer, der später nicht verwendet werden kann
Core::Logger::instance().warning(
    "Ungültige Parameter für sofortigen DMABUF-Buffer",
    "Wayland"
);
return;
}

// Überprüfe, ob das Format unterstützt wird
if (!isFormatSupported(params.format)) {
Core::Logger::instance().warning(
    QString("Nicht unterstütztes Format für sofortigen DMABUF-Buffer: 0x%1").arg(params.format, 0, 16),
    "Wayland"
);
return;
}

// Überprüfe, ob die Format-Modifier-Kombination unterstützt wird
if (!params.modifiers.isEmpty()) {
bool modifierSupported = false;

for (uint64_t modifier : params.modifiers) {
    if (isFormatModifierSupported(params.format, modifier)) {
        modifierSupported = true;
        break;
    }
}

if (!modifierSupported) {
    Core::Logger::instance().warning(
        QString("Nicht unterstützte Format-Modifier-Kombination für sofortigen DMABUF-Buffer: Format=0x%1, Modifier=0x%2")
            .arg(params.format, 0, 16)
            .arg(params.modifiers[0], 0, 16),
        "Wayland"
    );
    return;
}
}

// Erstelle ein neues buffer Objekt für den Client
int version = wl_resource_get_version(params_resource);
wl_client* client = wl_resource_get_client(params_resource);
QWaylandResource bufferResource = QWaylandResource::fromClient(client, wl_buffer_interface, 1, buffer_id);

// Erstelle einen LinuxDmabufBuffer
LinuxDmabufBuffer* buffer = new LinuxDmabufBuffer(bufferResource, params, this);

// Füge den Puffer zur Liste hinzu
addBuffer(buffer);
}

wl_listener LinuxDmabufProtocol::resource_destroy_params_listener[1];

const QVector<uint32_t>& LinuxDmabufProtocol::supportedFormats() const
{
return m_supportedFormats;
}

const QVector<FormatModifier>& LinuxDmabufProtocol::supportedFormatModifiers() const
{
return m_supportedFormatModifiers;
}

gbm_device* LinuxDmabufProtocol::gbmDevice() const
{
return m_gbmDevice;
}

int LinuxDmabufProtocol::drmFd() const
{
return m_drmFd;
}

EGLDisplay LinuxDmabufProtocol::eglDisplay() const
{
return m_eglDisplay;
}

const EGLFunctions& LinuxDmabufProtocol::eglFunctions() const
{
return m_eglFunctions;
}

} // namespace Wayland
} // namespace VivoX
