#include "VivoXSystem.h"

#include <QDebug>

int main(int argc, char *argv[])
{
    // Set up logging
    qSetMessagePattern("[%{time yyyy-MM-dd hh:mm:ss.zzz}] [%{type}] %{message}");

    // Create and initialize the VivoX system
    VivoX::VivoXSystem system;
    
    if (!system.initialize(argc, argv)) {
        qCritical() << "Failed to initialize VivoX system";
        return 1;
    }
    
    // Run the system
    int result = system.run();
    
    // Clean up
    system.shutdown();
    
    return result;
}
