#pragma once

#include <QObject>
#include <QString>
#include <QVariantMap>

namespace VivoX::System {

/**
 * @brief The MediaInfo class contains information about media playback.
 */
class MediaInfo {
public:
    QString playerId;     ///< Identifier for the media player
    QString playerName;   ///< Name of the media player
    QString title;        ///< Title of the media
    QString artist;       ///< Artist of the media
    QString album;        ///< Album of the media
    QString artUrl;       ///< URL to the album art
    qint64 length;        ///< Length of the media in microseconds
    qint64 position;      ///< Current position in microseconds
    double volume;        ///< Current volume (0.0 to 1.0)
    bool canPlay;         ///< Whether the player can play
    bool canPause;        ///< Whether the player can pause
    bool canSeek;         ///< Whether the player can seek
    bool canGoNext;       ///< Whether the player can go to the next track
    bool canGoPrevious;   ///< Whether the player can go to the previous track
    bool isPlaying;       ///< Whether the player is currently playing
    bool shuffle;         ///< Whether shuffle is enabled
    QString loopStatus;   ///< Loop status (None, Track, Playlist)
};

/**
 * @brief The MediaController class controls media playback.
 * 
 * It provides an interface to MPRIS-compatible media players via D-Bus
 * for controlling media playback and retrieving media information.
 */
class MediaController : public QObject {
    Q_OBJECT

public:
    explicit MediaController(QObject *parent = nullptr);
    ~MediaController();

    /**
     * @brief Initialize the media controller
     * @return True if initialization was successful
     */
    bool initialize();

    /**
     * @brief Get a list of all available media players
     * @return List of player IDs
     */
    QStringList getAvailablePlayers() const;

    /**
     * @brief Get information about the current media
     * @param playerId The player ID, or empty for the active player
     * @return The media info
     */
    MediaInfo getMediaInfo(const QString &playerId = QString()) const;

    /**
     * @brief Play media
     * @param playerId The player ID, or empty for the active player
     * @return True if successful
     */
    bool play(const QString &playerId = QString());

    /**
     * @brief Pause media
     * @param playerId The player ID, or empty for the active player
     * @return True if successful
     */
    bool pause(const QString &playerId = QString());

    /**
     * @brief Toggle play/pause
     * @param playerId The player ID, or empty for the active player
     * @return True if successful
     */
    bool playPause(const QString &playerId = QString());

    /**
     * @brief Stop media
     * @param playerId The player ID, or empty for the active player
     * @return True if successful
     */
    bool stop(const QString &playerId = QString());

    /**
     * @brief Go to the next track
     * @param playerId The player ID, or empty for the active player
     * @return True if successful
     */
    bool next(const QString &playerId = QString());

    /**
     * @brief Go to the previous track
     * @param playerId The player ID, or empty for the active player
     * @return True if successful
     */
    bool previous(const QString &playerId = QString());

    /**
     * @brief Seek to a position
     * @param position The position in microseconds
     * @param playerId The player ID, or empty for the active player
     * @return True if successful
     */
    bool seek(qint64 position, const QString &playerId = QString());

    /**
     * @brief Set the volume
     * @param volume The volume (0.0 to 1.0)
     * @param playerId The player ID, or empty for the active player
     * @return True if successful
     */
    bool setVolume(double volume, const QString &playerId = QString());

    /**
     * @brief Set shuffle mode
     * @param shuffle Whether to enable shuffle
     * @param playerId The player ID, or empty for the active player
     * @return True if successful
     */
    bool setShuffle(bool shuffle, const QString &playerId = QString());

    /**
     * @brief Set loop status
     * @param loopStatus The loop status (None, Track, Playlist)
     * @param playerId The player ID, or empty for the active player
     * @return True if successful
     */
    bool setLoopStatus(const QString &loopStatus, const QString &playerId = QString());

signals:
    /**
     * @brief Signal emitted when a player is added
     * @param playerId The player ID
     */
    void playerAdded(const QString &playerId);

    /**
     * @brief Signal emitted when a player is removed
     * @param playerId The player ID
     */
    void playerRemoved(const QString &playerId);

    /**
     * @brief Signal emitted when the active player changes
     * @param playerId The player ID
     */
    void activePlayerChanged(const QString &playerId);

    /**
     * @brief Signal emitted when media information changes
     * @param info The updated media info
     */
    void mediaInfoChanged(const MediaInfo &info);

private:
    // Map of player ID to media info
    QHash<QString, MediaInfo> m_mediaInfo;
    
    // Active player ID
    QString m_activePlayerId;
    
    // Get the player ID to use
    QString getPlayerIdToUse(const QString &playerId) const;
    
    // Update media information from D-Bus
    void updateMediaInfo();
};

} // namespace VivoX::System
