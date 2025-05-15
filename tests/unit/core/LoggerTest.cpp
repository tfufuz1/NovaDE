#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include "core/logging/Logger.h"

using namespace VivoX::Core::Logging;
using namespace testing;

class LoggerTest : public Test {
protected:
    void SetUp() override {
        // Create a test logger
        Logger::initialize("test_log.txt", LogLevel::DEBUG);
    }

    void TearDown() override {
        Logger::shutdown();
        // Clean up test log file
        std::remove("test_log.txt");
    }
};

TEST_F(LoggerTest, InitializationTest) {
    EXPECT_TRUE(Logger::isInitialized());
}

TEST_F(LoggerTest, LogLevelTest) {
    EXPECT_EQ(Logger::getLogLevel(), LogLevel::DEBUG);
    
    Logger::setLogLevel(LogLevel::INFO);
    EXPECT_EQ(Logger::getLogLevel(), LogLevel::INFO);
    
    Logger::setLogLevel(LogLevel::WARNING);
    EXPECT_EQ(Logger::getLogLevel(), LogLevel::WARNING);
    
    Logger::setLogLevel(LogLevel::ERROR);
    EXPECT_EQ(Logger::getLogLevel(), LogLevel::ERROR);
    
    Logger::setLogLevel(LogLevel::CRITICAL);
    EXPECT_EQ(Logger::getLogLevel(), LogLevel::CRITICAL);
    
    Logger::setLogLevel(LogLevel::TRACE);
    EXPECT_EQ(Logger::getLogLevel(), LogLevel::TRACE);
}

TEST_F(LoggerTest, LoggingTest) {
    // Redirect stdout to capture log output
    testing::internal::CaptureStdout();
    
    Logger::trace("This is a trace message");
    Logger::debug("This is a debug message");
    Logger::info("This is an info message");
    Logger::warning("This is a warning message");
    Logger::error("This is an error message");
    Logger::critical("This is a critical message");
    
    std::string output = testing::internal::GetCapturedStdout();
    
    // Check that all messages were logged
    EXPECT_THAT(output, HasSubstr("TRACE"));
    EXPECT_THAT(output, HasSubstr("DEBUG"));
    EXPECT_THAT(output, HasSubstr("INFO"));
    EXPECT_THAT(output, HasSubstr("WARNING"));
    EXPECT_THAT(output, HasSubstr("ERROR"));
    EXPECT_THAT(output, HasSubstr("CRITICAL"));
    
    EXPECT_THAT(output, HasSubstr("This is a trace message"));
    EXPECT_THAT(output, HasSubstr("This is a debug message"));
    EXPECT_THAT(output, HasSubstr("This is an info message"));
    EXPECT_THAT(output, HasSubstr("This is a warning message"));
    EXPECT_THAT(output, HasSubstr("This is an error message"));
    EXPECT_THAT(output, HasSubstr("This is a critical message"));
}

TEST_F(LoggerTest, LogLevelFilteringTest) {
    Logger::setLogLevel(LogLevel::WARNING);
    
    // Redirect stdout to capture log output
    testing::internal::CaptureStdout();
    
    Logger::trace("This is a trace message");
    Logger::debug("This is a debug message");
    Logger::info("This is an info message");
    Logger::warning("This is a warning message");
    Logger::error("This is an error message");
    Logger::critical("This is a critical message");
    
    std::string output = testing::internal::GetCapturedStdout();
    
    // Check that only WARNING and above messages were logged
    EXPECT_THAT(output, Not(HasSubstr("TRACE")));
    EXPECT_THAT(output, Not(HasSubstr("DEBUG")));
    EXPECT_THAT(output, Not(HasSubstr("INFO")));
    EXPECT_THAT(output, HasSubstr("WARNING"));
    EXPECT_THAT(output, HasSubstr("ERROR"));
    EXPECT_THAT(output, HasSubstr("CRITICAL"));
    
    EXPECT_THAT(output, Not(HasSubstr("This is a trace message")));
    EXPECT_THAT(output, Not(HasSubstr("This is a debug message")));
    EXPECT_THAT(output, Not(HasSubstr("This is an info message")));
    EXPECT_THAT(output, HasSubstr("This is a warning message"));
    EXPECT_THAT(output, HasSubstr("This is an error message"));
    EXPECT_THAT(output, HasSubstr("This is a critical message"));
}

TEST_F(LoggerTest, ThreadIdTest) {
    // Redirect stdout to capture log output
    testing::internal::CaptureStdout();
    
    Logger::info("Test thread ID");
    
    std::string output = testing::internal::GetCapturedStdout();
    
    // Check that thread ID is included in log message
    EXPECT_THAT(output, ContainsRegex("\\[TID:[0-9]+\\]"));
}

TEST_F(LoggerTest, TimestampFormatTest) {
    // Redirect stdout to capture log output
    testing::internal::CaptureStdout();
    
    Logger::info("Test timestamp format");
    
    std::string output = testing::internal::GetCapturedStdout();
    
    // Check that timestamp is in ISO 8601 format (YYYY-MM-DDTHH:MM:SS.sssZ)
    EXPECT_THAT(output, ContainsRegex("\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}\\.\\d{3}Z"));
}

TEST_F(LoggerTest, FlushTest) {
    // Test that flush works without crashing
    Logger::info("Message before flush");
    Logger::flush();
    Logger::info("Message after flush");
    
    // This test just verifies that flush() doesn't crash
    SUCCEED();
}

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
