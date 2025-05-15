#include "MediaController.h"

#include <QDebug>
#include <QDBusConnection>
#include <QDBusInterface>
#include <QDBusReply>
#include <QDBusConnectionInterface>
#include <QDBusMessage>
#include <QDBusObjectPath>
#include <QTimer>

namespace VivoX::System {

MediaController::MediaController(QObject *parent)
    : QObject(parent)
{
    qDebug() << "MediaController created";
}

MediaController::~MediaController()
{
    qDebug() << "MediaController destroyed";
}

bool MediaController::initialize()
{
    // Connect to the session bus
    QDBusConnection sessionBus = QDBusConnection::sessionBus();
    
    if (!sessionBus.isConnected()) {
        qWarning() << "Failed to connect to session bus";
        return false;
    }
    
    // Connect to DBus signals for player changes
    sessionBus.connect(
        "org.freedesktop.DBus", 
        "/org/freedesktop/DBus", 
        "org.freedesktop.DBus", 
        "NameOwnerChanged", 
        this, 
        SLOT(updateMediaInfo())
    );
    
    // Start a timer to periodically update media information
    QTimer *timer = new QTimer(this);
    connect(timer, &QTimer::timeout, this, &MediaController::updateMediaInfo);
    timer->start(2000); // Update every 2 seconds
    
    // Initial update
    updateMediaInfo();
    
    qDebug() << "MediaController initialized";
    return true;
}

QStringList MediaController::getAvailablePlayers() const
{
    return m_mediaInfo.keys();
}

MediaInfo MediaController::getMediaInfo(const QString &playerId) const
{
    QString id = getPlayerIdToUse(playerId);
    
    if (id.isEmpty() || !m_mediaInfo.contains(id)) {
        return MediaInfo();
    }
    
    return m_mediaInfo[id];
}

bool MediaController::play(const QString &playerId)
{
    QString id = getPlayerIdToUse(playerId);
    
    if (id.isEmpty()) {
        qWarning() << "No player available for play action";
        return false;
    }
    
    QDBusConnection sessionBus = QDBusConnection::sessionBus();
    QDBusInterface mprisInterface(
        id, 
        "/org/mpris/MediaPlayer2", 
        "org.mpris.MediaPlayer2.Player", 
        sessionBus
    );
    
    if (!mprisInterface.isValid()) {
        qWarning() << "Failed to create MPRIS interface for player:" << id;
        return false;
    }
    
    QDBusReply<void> reply = mprisInterface.call("Play");
    
    if (!reply.isValid()) {
        qWarning() << "Failed to call Play:" << reply.error().message();
        return false;
    }
    
    // Update media info after action
    QTimer::singleShot(100, this, &MediaController::updateMediaInfo);
    
    return true;
}

bool MediaController::pause(const QString &playerId)
{
    QString id = getPlayerIdToUse(playerId);
    
    if (id.isEmpty()) {
        qWarning() << "No player available for pause action";
        return false;
    }
    
    QDBusConnection sessionBus = QDBusConnection::sessionBus();
    QDBusInterface mprisInterface(
        id, 
        "/org/mpris/MediaPlayer2", 
        "org.mpris.MediaPlayer2.Player", 
        sessionBus
    );
    
    if (!mprisInterface.isValid()) {
        qWarning() << "Failed to create MPRIS interface for player:" << id;
        return false;
    }
    
    QDBusReply<void> reply = mprisInterface.call("Pause");
    
    if (!reply.isValid()) {
        qWarning() << "Failed to call Pause:" << reply.error().message();
        return false;
    }
    
    // Update media info after action
    QTimer::singleShot(100, this, &MediaController::updateMediaInfo);
    
    return true;
}

bool MediaController::playPause(const QString &playerId)
{
    QString id = getPlayerIdToUse(playerId);
    
    if (id.isEmpty()) {
        qWarning() << "No player available for playPause action";
        return false;
    }
    
    QDBusConnection sessionBus = QDBusConnection::sessionBus();
    QDBusInterface mprisInterface(
        id, 
        "/org/mpris/MediaPlayer2", 
        "org.mpris.MediaPlayer2.Player", 
        sessionBus
    );
    
    if (!mprisInterface.isValid()) {
        qWarning() << "Failed to create MPRIS interface for player:" << id;
        return false;
    }
    
    QDBusReply<void> reply = mprisInterface.call("PlayPause");
    
    if (!reply.isValid()) {
        qWarning() << "Failed to call PlayPause:" << reply.error().message();
        return false;
    }
    
    // Update media info after action
    QTimer::singleShot(100, this, &MediaController::updateMediaInfo);
    
    return true;
}

bool MediaController::stop(const QString &playerId)
{
    QString id = getPlayerIdToUse(playerId);
    
    if (id.isEmpty()) {
        qWarning() << "No player available for stop action";
        return false;
    }
    
    QDBusConnection sessionBus = QDBusConnection::sessionBus();
    QDBusInterface mprisInterface(
        id, 
        "/org/mpris/MediaPlayer2", 
        "org.mpris.MediaPlayer2.Player", 
        sessionBus
    );
    
    if (!mprisInterface.isValid()) {
        qWarning() << "Failed to create MPRIS interface for player:" << id;
        return false;
    }
    
    QDBusReply<void> reply = mprisInterface.call("Stop");
    
    if (!reply.isValid()) {
        qWarning() << "Failed to call Stop:" << reply.error().message();
        return false;
    }
    
    // Update media info after action
    QTimer::singleShot(100, this, &MediaController::updateMediaInfo);
    
    return true;
}

bool MediaController::next(const QString &playerId)
{
    QString id = getPlayerIdToUse(playerId);
    
    if (id.isEmpty()) {
        qWarning() << "No player available for next action";
        return false;
    }
    
    QDBusConnection sessionBus = QDBusConnection::sessionBus();
    QDBusInterface mprisInterface(
        id, 
        "/org/mpris/MediaPlayer2", 
        "org.mpris.MediaPlayer2.Player", 
        sessionBus
    );
    
    if (!mprisInterface.isValid()) {
        qWarning() << "Failed to create MPRIS interface for player:" << id;
        return false;
    }
    
    QDBusReply<void> reply = mprisInterface.call("Next");
    
    if (!reply.isValid()) {
        qWarning() << "Failed to call Next:" << reply.error().message();
        return false;
    }
    
    // Update media info after action
    QTimer::singleShot(100, this, &MediaController::updateMediaInfo);
    
    return true;
}

bool MediaController::previous(const QString &playerId)
{
    QString id = getPlayerIdToUse(playerId);
    
    if (id.isEmpty()) {
        qWarning() << "No player available for previous action";
        return false;
    }
    
    QDBusConnection sessionBus = QDBusConnection::sessionBus();
    QDBusInterface mprisInterface(
        id, 
        "/org/mpris/MediaPlayer2", 
        "org.mpris.MediaPlayer2.Player", 
        sessionBus
    );
    
    if (!mprisInterface.isValid()) {
        qWarning() << "Failed to create MPRIS interface for player:" << id;
        return false;
    }
    
    QDBusReply<void> reply = mprisInterface.call("Previous");
    
    if (!reply.isValid()) {
        qWarning() << "Failed to call Previous:" << reply.error().message();
        return false;
    }
    
    // Update media info after action
    QTimer::singleShot(100, this, &MediaController::updateMediaInfo);
    
    return true;
}

bool MediaController::seek(qint64 position, const QString &playerId)
{
    QString id = getPlayerIdToUse(playerId);
    
    if (id.isEmpty()) {
        qWarning() << "No player available for seek action";
        return false;
    }
    
    QDBusConnection sessionBus = QDBusConnection::sessionBus();
    QDBusInterface mprisInterface(
        id, 
        "/org/mpris/MediaPlayer2", 
        "org.mpris.MediaPlayer2.Player", 
        sessionBus
    );
    
    if (!mprisInterface.isValid()) {
        qWarning() << "Failed to create MPRIS interface for player:" << id;
        return false;
    }
    
    QDBusReply<void> reply = mprisInterface.call("SetPosition", QDBusObjectPath("/org/mpris/MediaPlayer2"), position);
    
    if (!reply.isValid()) {
        qWarning() << "Failed to call SetPosition:" << reply.error().message();
        return false;
    }
    
    // Update media info after action
    QTimer::singleShot(100, this, &MediaController::updateMediaInfo);
    
    return true;
}

bool MediaController::setVolume(double volume, const QString &playerId)
{
    QString id = getPlayerIdToUse(playerId);
    
    if (id.isEmpty()) {
        qWarning() << "No player available for setVolume action";
        return false;
    }
    
    // Ensure volume is within valid range
    volume = qBound(0.0, volume, 1.0);
    
    QDBusConnection sessionBus = QDBusConnection::sessionBus();
    QDBusInterface mprisInterface(
        id, 
        "/org/mpris/MediaPlayer2", 
        "org.mpris.MediaPlayer2.Player", 
        sessionBus
    );
    
    if (!mprisInterface.isValid()) {
        qWarning() << "Failed to create MPRIS interface for player:" << id;
        return false;
    }
    
    QDBusReply<void> reply = mprisInterface.call("Volume", volume);
    
    if (!reply.isValid()) {
        qWarning() << "Failed to call Volume:" << reply.error().message();
        return false;
    }
    
    // Update media info after action
    QTimer::singleShot(100, this, &MediaController::updateMediaInfo);
    
    return true;
}

bool MediaController::setShuffle(bool shuffle, const QString &playerId)
{
    QString id = getPlayerIdToUse(playerId);
    
    if (id.isEmpty()) {
        qWarning() << "No player available for setShuffle action";
        return false;
    }
    
    QDBusConnection sessionBus = QDBusConnection::sessionBus();
    QDBusInterface mprisInterface(
        id, 
        "/org/mpris/MediaPlayer2", 
        "org.mpris.MediaPlayer2.Player", 
        sessionBus
    );
    
    if (!mprisInterface.isValid()) {
        qWarning() << "Failed to create MPRIS interface for player:" << id;
        return false;
    }
    
    QDBusReply<void> reply = mprisInterface.call("Shuffle", shuffle);
    
    if (!reply.isValid()) {
        qWarning() << "Failed to call Shuffle:" << reply.error().message();
        return false;
    }
    
    // Update media info after action
    QTimer::singleShot(100, this, &MediaController::updateMediaInfo);
    
    return true;
}

bool MediaController::setLoopStatus(const QString &loopStatus, const QString &playerId)
{
    QString id = getPlayerIdToUse(playerId);
    
    if (id.isEmpty()) {
        qWarning() << "No player available for setLoopStatus action";
        return false;
    }
    
    // Validate loop status
    if (loopStatus != "None" && loopStatus != "Track" && loopStatus != "Playlist") {
        qWarning() << "Invalid loop status:" << loopStatus;
        return false;
    }
    
    QDBusConnection sessionBus = QDBusConnection::sessionBus();
    QDBusInterface mprisInterface(
        id, 
        "/org/mpris/MediaPlayer2", 
        "org.mpris.MediaPlayer2.Player", 
        sessionBus
    );
    
    if (!mprisInterface.isValid()) {
        qWarning() << "Failed to create MPRIS interface for player:" << id;
        return false;
    }
    
    QDBusReply<void> reply = mprisInterface.call("LoopStatus", loopStatus);
    
    if (!reply.isValid()) {
        qWarning() << "Failed to call LoopStatus:" << reply.error().message();
        return false;
    }
    
    // Update media info after action
    QTimer::singleShot(100, this, &MediaController::updateMediaInfo);
    
    return true;
}

QString MediaController::getPlayerIdToUse(const QString &playerId) const
{
    // If a specific player ID is provided, use it
    if (!playerId.isEmpty() && m_mediaInfo.contains(playerId)) {
        return playerId;
    }
    
    // Otherwise, use the active player
    if (!m_activePlayerId.isEmpty() && m_mediaInfo.contains(m_activePlayerId)) {
        return m_activePlayerId;
    }
    
    // If no active player, use the first available player
    if (!m_mediaInfo.isEmpty()) {
        return m_mediaInfo.keys().first();
    }
    
    // No players available
    return QString();
}

void MediaController::updateMediaInfo()
{
    QDBusConnection sessionBus = QDBusConnection::sessionBus();
    
    if (!sessionBus.isConnected()) {
        qWarning() << "Failed to connect to session bus";
        return;
    }
    
    // Get all services on the session bus
    QDBusReply<QStringList> reply = sessionBus.interface()->registeredServiceNames();
    
    if (!reply.isValid()) {
        qWarning() << "Failed to get registered service names:" << reply.error().message();
        return;
    }
    
    QStringList services = reply.value();
    QStringList mprisServices;
    
    // Filter for MPRIS services
    for (const QString &service : services) {
        if (service.startsWith("org.mpris.MediaPlayer2.")) {
            mprisServices.append(service);
        }
    }
    
    // Check for removed players
    QStringList currentPlayers = m_mediaInfo.keys();
    for (const QString &player : currentPlayers) {
        if (!mprisServices.contains(player)) {
            m_mediaInfo.remove(player);
            emit playerRemoved(player);
            
            // If this was the active player, clear it
            if (m_activePlayerId == player) {
                m_activePlayerId.clear();
            }
        }
    }
    
    // Update info for each MPRIS service
    for (const QString &service : mprisServices) {
        QDBusInterface mprisInterface(
            service, 
            "/org/mpris/MediaPlayer2", 
            "org.mpris.MediaPlayer2.Player", 
            sessionBus
        );
        
        if (!mprisInterface.isValid()) {
            qWarning() << "Failed to create MPRIS interface for service:" << service;
            continue;
        }
        
        // Get player identity
        QDBusInterface identityInterface(
            service, 
            "/org/mpris/MediaPlayer2", 
            "org.mpris.MediaPlayer2", 
            sessionBus
        );
        
        QString playerName;
        if (identityInterface.isValid()) {
            QDBusReply<QString> identityReply = identityInterface.call("Identity");
            if (identityReply.isValid()) {
                playerName = identityReply.value();
            }
        }
        
        // Get player properties
        QDBusMessage metadataMsg = QDBusMessage::createMethodCall(
            service,
            "/org/mpris/MediaPlayer2",
            "org.freedesktop.DBus.Properties",
            "Get"
        );
        metadataMsg << "org.mpris.MediaPlayer2.Player" << "Metadata";
        QDBusReply<QVariant> metadataReply = sessionBus.call(metadataMsg);
        
        QDBusMessage playbackStatusMsg = QDBusMessage::createMethodCall(
            service,
            "/org/mpris/MediaPlayer2",
            "org.freedesktop.DBus.Properties",
            "Get"
        );
        playbackStatusMsg << "org.mpris.MediaPlayer2.Player" << "PlaybackStatus";
        QDBusReply<QVariant> playbackStatusReply = sessionBus.call(playbackStatusMsg);
        
        QDBusMessage volumeMsg = QDBusMessage::createMethodCall(
            service,
            "/org/mpris/MediaPlayer2",
            "org.freedesktop.DBus.Properties",
            "Get"
        );
        volumeMsg << "org.mpris.MediaPlayer2.Player" << "Volume";
        QDBusReply<QVariant> volumeReply = sessionBus.call(volumeMsg);
        
        QDBusMessage shuffleMsg = QDBusMessage::createMethodCall(
            service,
            "/org/mpris/MediaPlayer2",
            "org.freedesktop.DBus.Properties",
            "Get"
        );
        shuffleMsg << "org.mpris.MediaPlayer2.Player" << "Shuffle";
        QDBusReply<QVariant> shuffleReply = sessionBus.call(shuffleMsg);
        
        QDBusMessage loopStatusMsg = QDBusMessage::createMethodCall(
            service,
            "/org/mpris/MediaPlayer2",
            "org.freedesktop.DBus.Properties",
            "Get"
        );
        loopStatusMsg << "org.mpris.MediaPlayer2.Player" << "LoopStatus";
        QDBusReply<QVariant> loopStatusReply = sessionBus.call(loopStatusMsg);
        
        QDBusMessage canControlMsg = QDBusMessage::createMethodCall(
            service,
            "/org/mpris/MediaPlayer2",
            "org.freedesktop.DBus.Properties",
            "Get"
        );
        canControlMsg << "org.mpris.MediaPlayer2.Player" << "CanControl";
        QDBusReply<QVariant> canControlReply = sessionBus.call(canControlMsg);
        
        QDBusMessage canPlayMsg = QDBusMessage::createMethodCall(
            service,
            "/org/mpris/MediaPlayer2",
            "org.freedesktop.DBus.Properties",
            "Get"
        );
        canPlayMsg << "org.mpris.MediaPlayer2.Player" << "CanPlay";
        QDBusReply<QVariant> canPlayReply = sessionBus.call(canPlayMsg);
        
        QDBusMessage canPauseMsg = QDBusMessage::createMethodCall(
            service,
            "/org/mpris/MediaPlayer2",
            "org.freedesktop.DBus.Properties",
            "Get"
        );
        canPauseMsg << "org.mpris.MediaPlayer2.Player" << "CanPause";
        QDBusReply<QVariant> canPauseReply = sessionBus.call(canPauseMsg);
        
        QDBusMessage canSeekMsg = QDBusMessage::createMethodCall(
            service,
            "/org/mpris/MediaPlayer2",
            "org.freedesktop.DBus.Properties",
            "Get"
        );
        canSeekMsg << "org.mpris.MediaPlayer2.Player" << "CanSeek";
        QDBusReply<QVariant> canSeekReply = sessionBus.call(canSeekMsg);
        
        QDBusMessage canGoNextMsg = QDBusMessage::createMethodCall(
            service,
            "/org/mpris/MediaPlayer2",
            "org.freedesktop.DBus.Properties",
            "Get"
        );
        canGoNextMsg << "org.mpris.MediaPlayer2.Player" << "CanGoNext";
        QDBusReply<QVariant> canGoNextReply = sessionBus.call(canGoNextMsg);
        
        QDBusMessage canGoPreviousMsg = QDBusMessage::createMethodCall(
            service,
            "/org/mpris/MediaPlayer2",
            "org.freedesktop.DBus.Properties",
            "Get"
        );
        canGoPreviousMsg << "org.mpris.MediaPlayer2.Player" << "CanGoPrevious";
        QDBusReply<QVariant> canGoPreviousReply = sessionBus.call(canGoPreviousMsg);
        
        QDBusMessage positionMsg = QDBusMessage::createMethodCall(
            service,
            "/org/mpris/MediaPlayer2",
            "org.freedesktop.DBus.Properties",
            "Get"
        );
        positionMsg << "org.mpris.MediaPlayer2.Player" << "Position";
        QDBusReply<QVariant> positionReply = sessionBus.call(positionMsg);
        
        // Create or update media info
        MediaInfo info;
        info.playerId = service;
        info.playerName = playerName.isEmpty() ? service.mid(23) : playerName; // Remove "org.mpris.MediaPlayer2." prefix
        
        // Set defaults
        info.canPlay = false;
        info.canPause = false;
        info.canSeek = false;
        info.canGoNext = false;
        info.canGoPrevious = false;
        info.isPlaying = false;
        info.shuffle = false;
        info.loopStatus = "None";
        info.volume = 1.0;
        info.length = 0;
        info.position = 0;
        
        // Update with actual values if available
        if (metadataReply.isValid()) {
            QVariantMap metadata = qdbus_cast<QVariantMap>(metadataReply.value());
            
            if (metadata.contains("xesam:title")) {
                info.title = metadata["xesam:title"].toString();
            }
            
            if (metadata.contains("xesam:artist")) {
                QStringList artists = metadata["xesam:artist"].toStringList();
                info.artist = artists.join(", ");
            }
            
            if (metadata.contains("xesam:album")) {
                info.album = metadata["xesam:album"].toString();
            }
            
            if (metadata.contains("mpris:artUrl")) {
                info.artUrl = metadata["mpris:artUrl"].toString();
            }
            
            if (metadata.contains("mpris:length")) {
                info.length = metadata["mpris:length"].toLongLong();
            }
        }
        
        if (playbackStatusReply.isValid()) {
            QString status = playbackStatusReply.value().toString();
            info.isPlaying = (status == "Playing");
        }
        
        if (volumeReply.isValid()) {
            info.volume = volumeReply.value().toDouble();
        }
        
        if (shuffleReply.isValid()) {
            info.shuffle = shuffleReply.value().toBool();
        }
        
        if (loopStatusReply.isValid()) {
            info.loopStatus = loopStatusReply.value().toString();
        }
        
        if (canControlReply.isValid() && canControlReply.value().toBool()) {
            if (canPlayReply.isValid()) {
                info.canPlay = canPlayReply.value().toBool();
            }
            
            if (canPauseReply.isValid()) {
                info.canPause = canPauseReply.value().toBool();
            }
            
            if (canSeekReply.isValid()) {
                info.canSeek = canSeekReply.value().toBool();
            }
            
            if (canGoNextReply.isValid()) {
                info.canGoNext = canGoNextReply.value().toBool();
            }
            
            if (canGoPreviousReply.isValid()) {
                info.canGoPrevious = canGoPreviousReply.value().toBool();
            }
        }
        
        if (positionReply.isValid()) {
            info.position = positionReply.value().toLongLong();
        }
        
        // Check if this is a new player
        bool isNewPlayer = !m_mediaInfo.contains(service);
        
        // Update the media info
        m_mediaInfo[service] = info;
        
        // Emit signals
        if (isNewPlayer) {
            emit playerAdded(service);
            
            // If no active player, set this as active
            if (m_activePlayerId.isEmpty()) {
                m_activePlayerId = service;
                emit activePlayerChanged(service);
            }
        }
        
        // If this is the active player, emit mediaInfoChanged
        if (service == m_activePlayerId) {
            emit mediaInfoChanged(info);
        }
        
        // If this player is playing and the active player is not, make this the active player
        if (info.isPlaying && !m_mediaInfo[m_activePlayerId].isPlaying) {
            m_activePlayerId = service;
            emit activePlayerChanged(service);
            emit mediaInfoChanged(info);
        }
    }
}

} // namespace VivoX::System
