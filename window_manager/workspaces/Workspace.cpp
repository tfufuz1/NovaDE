#include "Workspace.h"

#include <QDebug>

namespace VivoX::WindowManager {

Workspace::Workspace(QObject *parent)
    : QObject(parent)
    , m_active(false)
{
    qDebug() << "Workspace created";
}

Workspace::~Workspace()
{
    qDebug() << "Workspace destroyed:" << m_id;
}

QString Workspace::id() const
{
    return m_id;
}

void Workspace::setId(const QString &id)
{
    if (m_id != id) {
        m_id = id;
        emit idChanged(m_id);
    }
}

QString Workspace::name() const
{
    return m_name;
}

void Workspace::setName(const QString &name)
{
    if (m_name != name) {
        m_name = name;
        emit nameChanged(m_name);
        
        qDebug() << "Workspace name changed:" << m_id << m_name;
    }
}

QRect Workspace::geometry() const
{
    return m_geometry;
}

void Workspace::setGeometry(const QRect &geometry)
{
    if (m_geometry != geometry) {
        m_geometry = geometry;
        emit geometryChanged(m_geometry);
        
        qDebug() << "Workspace geometry changed:" << m_id << m_geometry;
    }
}

bool Workspace::isActive() const
{
    return m_active;
}

void Workspace::setActive(bool active)
{
    if (m_active != active) {
        m_active = active;
        emit activeChanged(m_active);
        
        qDebug() << "Workspace active state changed:" << m_id << m_active;
    }
}

} // namespace VivoX::WindowManager
