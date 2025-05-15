#pragma once

#include <QObject>
#include <QString>
#include <QList>

namespace VivoX::Core {

/**
 * @brief The TestRunner class provides functionality for testing system components.
 * 
 * It allows running tests on individual components or the entire system to verify
 * that everything is working correctly.
 */
class TestRunner : public QObject {
    Q_OBJECT

public:
    /**
     * @brief Struct representing a test result
     */
    struct TestResult {
        QString testName;     ///< Name of the test
        QString component;    ///< Component being tested
        bool success;         ///< Whether the test passed
        QString message;      ///< Additional message (error details if failed)
    };

    explicit TestRunner(QObject *parent = nullptr);
    ~TestRunner();

    /**
     * @brief Initialize the test runner
     * @return True if initialization was successful
     */
    bool initialize();

    /**
     * @brief Run all tests
     * @return List of test results
     */
    QList<TestResult> runAllTests();

    /**
     * @brief Run tests for a specific component
     * @param component The component to test
     * @return List of test results
     */
    QList<TestResult> runComponentTests(const QString &component);

    /**
     * @brief Get a summary of test results
     * @param results The test results
     * @return A summary string
     */
    QString getTestSummary(const QList<TestResult> &results);

signals:
    /**
     * @brief Signal emitted when a test starts
     * @param testName The name of the test
     * @param component The component being tested
     */
    void testStarted(const QString &testName, const QString &component);

    /**
     * @brief Signal emitted when a test completes
     * @param result The test result
     */
    void testCompleted(const TestResult &result);

    /**
     * @brief Signal emitted when all tests complete
     * @param results The test results
     */
    void testsCompleted(const QList<TestResult> &results);

private:
    // Run core component tests
    QList<TestResult> runCoreTests();

    // Run compositor component tests
    QList<TestResult> runCompositorTests();

    // Run window manager component tests
    QList<TestResult> runWindowManagerTests();

    // Run UI component tests
    QList<TestResult> runUITests();

    // Run input component tests
    QList<TestResult> runInputTests();

    // Run system component tests
    QList<TestResult> runSystemTests();

    // Run integration tests
    QList<TestResult> runIntegrationTests();

    // Helper method to create a test result
    TestResult createTestResult(const QString &testName, const QString &component, 
                               bool success, const QString &message = QString());
};

} // namespace VivoX::Core
