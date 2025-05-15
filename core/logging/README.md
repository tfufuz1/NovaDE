# VivoX Core Library - Logging System

This library provides a robust, flexible logging system for the VivoX Desktop Environment.

## Features
- Multiple log levels (DEBUG, INFO, WARNING, ERROR, CRITICAL)
- File and console output
- Thread-safe logging
- Log rotation
- Asynchronous logging for high-performance scenarios

## Usage
```cpp
#include <vivox/core/logging/Logger.h>

// Get a logger instance
auto logger = VivoX::Core::Logging::Logger::getInstance("MyComponent");

// Log messages at different levels
logger->debug("This is a debug message");
logger->info("This is an info message");
logger->warning("This is a warning message");
logger->error("This is an error message");
logger->critical("This is a critical message");

// Log with formatting
logger->info("User {} logged in from {}", username, ipAddress);

// Configure logger
VivoX::Core::Logging::LogConfig config;
config.setLogLevel(VivoX::Core::Logging::LogLevel::INFO);
config.setLogFile("/var/log/vivox/mycomponent.log");
config.setMaxFileSize(10 * 1024 * 1024); // 10 MB
config.setMaxBackupFiles(5);
config.setAsyncLogging(true);
logger->configure(config);
```
