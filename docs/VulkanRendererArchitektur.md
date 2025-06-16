# Architektur des NovaDE Vulkan-Rendering-Subsystems

## 1. Überblick

Dieses Dokument beschreibt die Architektur des Vulkan-Rendering-Subsystems für den NovaDE Compositor. Ziel ist es, eine explizite, performante und moderne Rendering-Pipeline bereitzustellen, die auf der Vulkan-API aufbaut. Der Renderer ist dafür verantwortlich, die vom Compositor verwalteten Fenster und Oberflächen darzustellen und zukünftig erweiterte grafische Effekte zu ermöglichen.

## 2. Hauptkomponenten und ihre Verantwortlichkeiten

Das Vulkan-Rendering-Subsystem ist in mehrere Module unterteilt, die jeweils spezifische Verantwortlichkeiten tragen:

*   **`error.rs`**: Definiert die benutzerdefinierten Fehlertypen (`VulkanError`) und den `Result`-Alias für das gesamte Vulkan-Modul, um eine konsistente Fehlerbehandlung zu gewährleisten.
*   **`instance.rs` (`VulkanInstance`)**: Verantwortlich für die Initialisierung der Vulkan-API durch Erstellung einer `VkInstance`. Dies beinhaltet das Laden von Instanz-Erweiterungen (z.B. für Wayland-Integration, Debugging) und die optionale Aktivierung von Validierungsschichten sowie die Einrichtung eines Debug-Messengers.
*   **`physical_device.rs` (`PhysicalDeviceInfo`)**: Beinhaltet die Logik zur Auswahl eines geeigneten `VkPhysicalDevice` (GPU). Bewertet Geräte basierend auf ihren Fähigkeiten, Features, Erweiterungen und Queue-Familien-Unterstützung (Grafik, Präsentation, Compute, Transfer). Speichert Informationen über das ausgewählte Gerät.
*   **`device.rs` (`LogicalDevice`, `Queues`)**: Erstellt das `VkDevice` (logisches Gerät) aus dem ausgewählten physischen Gerät. Aktiviert notwendige Geräte-Features und -Erweiterungen und ruft Handles für die benötigten `VkQueue`s ab.
*   **`allocator.rs` (`Allocator`)**: Kapselt den Vulkan Memory Allocator (VMA). Stellt Funktionen zur Verfügung, um Vulkan-Buffer (`VkBuffer`) und -Images (`VkImage`) mit optimierter Speicherverwaltung (unter Berücksichtigung der UMA-Architektur des Zielsystems) zu erstellen und zu zerstören.
*   **`sync_primitives.rs` (`FrameSyncPrimitives`)**: Definiert und verwaltet die Synchronisationsobjekte (`VkSemaphore`, `VkFence`), die pro Frame für die Koordination zwischen CPU und GPU in der Render-Schleife benötigt werden.
*   **`buffer_utils.rs`**: Enthält Hilfsfunktionen für Buffer-Operationen, insbesondere `create_and_fill_gpu_buffer` zum Erstellen von GPU-lokalen Buffern (Vertex, Index) unter Verwendung von Staging-Buffern.
*   **`texture.rs` (`Texture`)**: Verantwortlich für das Laden von Bilddateien, Erstellen von `VkImage`s, `VkImageView`s und `VkSampler`s. Implementiert Mipmap-Generierung und das Hochladen von Texturdaten auf die GPU (inkl. Staging-Buffer und Layout-Transitionen). Beinhaltet auch Logik zur Erstellung von Storage Images für Compute-Shader.
*   **`render_pass.rs` (`RenderPass`)**: Definiert den `VkRenderPass`, der die Struktur der Rendering-Operationen beschreibt, einschließlich der Attachments (Farb- und Tiefenpuffer) und deren Verwendung (Lade-/Speicheroperationen, Layouts) sowie Subpass-Abhängigkeiten.
*   **`framebuffer.rs`**: Enthält Funktionen zur Erstellung von `VkFramebuffer`-Objekten, die die konkreten `VkImageView`s (aus der Swapchain und für den Tiefenpuffer) an einen `VkRenderPass` binden.
*   **`pipeline.rs` (`GraphicsPipeline`, `PipelineLayout`, Shader-Ladefunktionen, Vertex-, UBO-, PushConstant-Strukturen)**: Dieses umfangreiche Modul ist zentral für die Definition der Rendering-Pipeline. Es beinhaltet:
    *   Laden von SPIR-V Shader-Dateien und Erstellen von `VkShaderModule`s.
    *   Definition von Datenstrukturen für Vertex-Input (`Vertex`), Uniform Buffer Objects (`UniformBufferObject`) und Push Constants (`GraphicsPushConstants`).
    *   Erstellung von `VkPipelineLayout` (definiert Descriptor-Set-Layouts und Push-Constant-Ranges).
    *   Erstellung der `VkPipeline` (Grafik-Pipeline) mit allen Konfigurationsstufen (Shader, Vertex-Input, Input-Assembly, Viewport/Scissor, Rasterisierung, Multisampling, Depth/Stencil, Color Blending, dynamische Zustände).
    *   Erstellung von Compute-Pipelines.
    *   Verwaltung des Tiefenpuffers (Erstellung von Image und View).
*   **`surface_swapchain.rs` (`SurfaceSwapchain`, `RawWindowHandleWrapper`)**: Integriert mit dem Wayland-System zur Erstellung einer `VkSurfaceKHR`. Verwaltet die `VkSwapchainKHR`, einschließlich Erstellung, Abruf der Swapchain-Images, Erstellung der zugehörigen `VkImageView`s und der Logik zur Swapchain-Rekreation bei Bedarf.
*   **`frame_renderer.rs` (`FrameRenderer`)**: Die zentrale Orchestrierungs-Einheit. Initialisiert und hält die meisten der oben genannten Vulkan-Objekte. Implementiert die Haupt-Render-Schleife (`draw_frame`), die die Synchronisation, das Abrufen von Swapchain-Images, die Command-Buffer-Aufzeichnung (Compute- und Grafik-Pass), die Submission an die Queues und die Präsentation der gerenderten Bilder steuert. Verwaltet auch den Pipeline-Cache.

## 3. Datenfluss (vereinfacht)

1.  **Initialisierung:** Alle Vulkan-Komponenten werden erstellt, Shader geladen, Pipelines konfiguriert, und Ressourcen (Texturen, initiale Buffer) werden auf die GPU hochgeladen.
2.  **Render-Schleife (`draw_frame`):**
    *   **CPU:** Wartet auf die GPU des vorherigen Frames. Aktualisiert Uniform Buffer und Push Constants mit dynamischen Daten für den aktuellen Frame.
    *   **GPU-Vorbereitung:** Akquiriert ein Swapchain-Image.
    *   **Command Buffer Recording:**
        *   **(Compute-Pass):**
            *   Input-Textur wird an den Compute-Shader gebunden.
            *   Compute-Shader wird ausgeführt (`vkCmdDispatch`), schreibt in ein Output-(Storage)-Image.
            *   Synchronisationsbarriere stellt sicher, dass Compute-Output für den Grafik-Pass bereit ist.
        *   **(Grafik-Pass):**
            *   Render Pass wird gestartet.
            *   Grafik-Pipeline und zugehörige Descriptor Sets (UBO, Ergebnis des Compute-Passes als Textur) werden gebunden.
            *   Vertex- und Index-Buffer werden gebunden.
            *   Push Constants werden gesendet.
            *   `vkCmdDrawIndexed` zeichnet die Geometrie.
            *   Render Pass wird beendet.
    *   **Submission:** Der Command Buffer wird an die Grafik-Queue übermittelt. Ein Fence signalisiert der CPU die Fertigstellung. Ein Semaphore signalisiert die Render-Fertigstellung für die Präsentation.
    *   **Präsentation:** Das gerenderte Swapchain-Image wird präsentiert.

## 4. Speicherverwaltung

*   Die Speicherallokation für Vulkan-Buffer und -Images wird durch den **Vulkan Memory Allocator (VMA)** über das `allocator.rs`-Modul gehandhabt.
*   Für das Zielsystem (AMD APU mit Unified Memory Architecture - UMA) werden Speichertypen bevorzugt, die sowohl `DEVICE_LOCAL` als auch `HOST_VISIBLE` sind, um explizite Staging-Buffer-Kopien für häufig aktualisierte Daten (wie Uniform Buffers via `MemoryUsage::CpuToGpu`) zu minimieren.
*   Für statische Daten (Vertex-/Index-Buffer, Texturen via `MemoryUsage::GpuOnly`) wird GPU-lokaler Speicher verwendet, wobei der initiale Upload über Staging-Buffer (`buffer_utils.rs` und `texture.rs`) erfolgt.

## 5. Synchronisation

*   **`VkFence`**: Wird verwendet, um die CPU mit der GPU zu synchronisieren, insbesondere um sicherzustellen, dass ein Frame vollständig gerendert wurde, bevor die CPU Ressourcen für den nächsten Frame wiederverwendet (`FrameRenderer::draw_frame`).
*   **`VkSemaphore`**: Dient der GPU-GPU-Synchronisation zwischen Queue-Operationen. Hauptsächlich verwendet zwischen der Swapchain-Image-Akquise und dem Beginn der Command-Buffer-Ausführung sowie zwischen dem Ende der Command-Buffer-Ausführung und der Präsentation des Bildes.
*   **`VkPipelineBarrier` (`VkImageMemoryBarrier`, `VkBufferMemoryBarrier`)**: Wird für feingranulare Synchronisation innerhalb eines Command Buffers verwendet, insbesondere für:
    *   Image-Layout-Transitionen (z.B. von `UNDEFINED` zu `TRANSFER_DST_OPTIMAL` zu `SHADER_READ_ONLY_OPTIMAL`).
    *   Sicherstellung der Datenkonsistenz zwischen Schreib- und Leseoperationen (z.B. zwischen Compute-Shader-Ausgabe und Grafik-Shader-Eingabe).

## 6. Fehlerbehandlung

*   Das Modul `error.rs` definiert eine zentrale `VulkanError`-Enum und einen `Result`-Typalias.
*   Alle Funktionen, die fehlschlagen können, geben diesen `Result`-Typ zurück.
*   Vulkan API-Fehler (`vk::Result`), VMA-Fehler (`vk_mem::Error`) und I/O-Fehler werden in `VulkanError` konvertiert.
*   Spezifische Fehlerfälle wie `VK_ERROR_OUT_OF_DATE_KHR` werden als `VulkanError::SwapchainOutOfDate` behandelt, um eine Swapchain-Rekreation auszulösen.
*   Logging über die `log`-Crate wird an kritischen Stellen und bei Fehlern verwendet. Der Vulkan Debug Messenger liefert detaillierte Validierungsfehler während der Entwicklung.

---
*Weitere Abschnitte (z.B. Wichtige Designentscheidungen, Zukünftige Erweiterungen) können später hinzugefügt werden.*
