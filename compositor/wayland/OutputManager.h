#pragma once

#include <memory>
#include <vector>
#include <string>
#include <map>
#include <mutex>

namespace VivoX {
    namespace Compositor {
        namespace Wayland {

            class WaylandCompositor;

            /**
             * @brief Output manager for the VivoX Desktop Environment
             *
             * This class is responsible for managing outputs (monitors) in the
             * Wayland compositor. It handles output detection, configuration,
             * layout management, and persistence.
             */
            class OutputManager {
            public:
                /**
                 * @brief Output information structure
                 */
                struct OutputInfo {
                    std::string name;           ///< Unique identifier for the output
                    std::string model;          ///< Model name of the display
                    std::string manufacturer;   ///< Manufacturer of the display
                    int x;                      ///< X position in the global output layout
                    int y;                      ///< Y position in the global output layout
                    int width;                  ///< Width in pixels
                    int height;                 ///< Height in pixels
                    int physicalWidth;          ///< Physical width in millimeters
                    int physicalHeight;         ///< Physical height in millimeters
                    float scale;                ///< HiDPI scale factor
                    int refreshRate;            ///< Refresh rate in mHz (60000 = 60Hz)
                    bool enabled;               ///< Whether the output is enabled
                    bool primary;               ///< Whether this is the primary output
                };

                /**
                 * @brief Get the singleton instance of the OutputManager
                 * @return Shared pointer to the OutputManager instance
                 */
                static std::shared_ptr<OutputManager> getInstance();

                /**
                 * @brief Initialize the output manager
                 * @param compositor Wayland compositor instance
                 * @return True if initialization was successful, false otherwise
                 */
                bool initialize(std::shared_ptr<WaylandCompositor> compositor);

                /**
                 * @brief Shutdown the output manager
                 */
                void shutdown();

                /**
                 * @brief Check if the output manager is initialized
                 * @return True if the output manager is initialized, false otherwise
                 */
                bool isInitialized() const;

                /**
                 * @brief Get all outputs
                 * @return Vector of all outputs
                 */
                std::vector<OutputInfo> getOutputs() const;

                /**
                 * @brief Get an output by name
                 * @param name Name of the output to get
                 * @return Output information, or nullptr if it wasn't found
                 */
                const OutputInfo* getOutput(const std::string& name) const;

                /**
                 * @brief Get the primary output
                 * @return Primary output information, or nullptr if no primary output is set
                 */
                const OutputInfo* getPrimaryOutput() const;

                /**
                 * @brief Set the primary output
                 * @param name Name of the output to set as primary
                 * @return True if the operation was successful, false otherwise
                 */
                bool setPrimaryOutput(const std::string& name);

                /**
                 * @brief Enable or disable an output
                 * @param name Name of the output to enable or disable
                 * @param enable Whether to enable or disable the output
                 * @return True if the operation was successful, false otherwise
                 */
                bool setOutputEnabled(const std::string& name, bool enable);

                /**
                 * @brief Set the position of an output
                 * @param name Name of the output to set position for
                 * @param x X position
                 * @param y Y position
                 * @return True if the operation was successful, false otherwise
                 */
                bool setOutputPosition(const std::string& name, int x, int y);

                /**
                 * @brief Set the scale of an output
                 * @param name Name of the output to set scale for
                 * @param scale Scale factor
                 * @return True if the operation was successful, false otherwise
                 */
                bool setOutputScale(const std::string& name, float scale);

                /**
                 * @brief Set the mode of an output
                 * @param name Name of the output to set mode for
                 * @param width Width in pixels
                 * @param height Height in pixels
                 * @param refreshRate Refresh rate in mHz
                 * @return True if the operation was successful, false otherwise
                 */
                bool setOutputMode(const std::string& name, int width, int height, int refreshRate);

                /**
                 * @brief Apply output configuration
                 * @return True if the operation was successful, false otherwise
                 */
                bool applyConfiguration();

                /**
                 * @brief Get the number of outputs
                 * @return Number of outputs
                 */
                int getOutputCount() const;

                /**
                 * @brief Check if an output exists
                 * @param name Name of the output to check
                 * @return True if the output exists, false otherwise
                 */
                bool outputExists(const std::string& name) const;

                /**
                 * @brief Load output configuration from file
                 * @return True if the operation was successful, false otherwise
                 */
                bool loadConfiguration();

                /**
                 * @brief Save output configuration to file
                 * @return True if the operation was successful, false otherwise
                 */
                bool saveConfiguration();

                /**
                 * @brief Auto-configure outputs in a sensible layout
                 * @return True if the operation was successful, false otherwise
                 */
                bool autoConfigureOutputs();

                /**
                 * @brief Get total desktop area
                 * @param x Output parameter for the left-most coordinate
                 * @param y Output parameter for the top-most coordinate
                 * @param width Output parameter for the total width
                 * @param height Output parameter for the total height
                 */
                void getTotalDesktopArea(int& x, int& y, int& width, int& height) const;

                /**
                 * @brief Get output at a specific position
                 * @param x X coordinate
                 * @param y Y coordinate
                 * @return Output information, or nullptr if no output is at that position
                 */
                const OutputInfo* getOutputAt(int x, int y) const;

                /**
                 * @brief Clone an output to another output
                 * @param sourceName Name of the output to clone from
                 * @param targetName Name of the output to clone to
                 * @return True if the operation was successful, false otherwise
                 */
                bool cloneOutput(const std::string& sourceName, const std::string& targetName);

            private:
                OutputManager();
                ~OutputManager();

                static std::shared_ptr<OutputManager> s_instance;
                static std::mutex s_instanceMutex;

                class Impl;
                std::unique_ptr<Impl> m_impl;
            };

        } // namespace Wayland
    } // namespace Compositor
} // namespace VivoX
