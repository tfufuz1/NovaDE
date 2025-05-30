# ADR-0001: Logging Strategy

**Status:** Accepted

**Date:** 2023-10-27

**Context:**

The need for a robust, performant, and configurable logging solution across all NovaDE components (compositor, shell, utilities) is paramount. Logging is crucial for:
*   Debugging issues during development and testing.
*   Monitoring system behavior and performance in real-time.
*   Providing diagnostics for user-reported problems.
*   Potentially, for security auditing in later stages.

Given the complexity of a desktop environment, with multiple processes and asynchronous events, a consistent and well-thought-out logging strategy is essential from the project's inception.

**Decision:**

We will adopt the following logging strategy for NovaDE:

1.  **Primary Logging Facade:** The `tracing` crate will be used as the primary logging facade. Its rich feature set, including structured logging, spans, and asynchronous support, makes it suitable for a complex project like NovaDE.
2.  **Subscriber Configuration:** `tracing-subscriber` will be utilized to configure log collection, filtering, and output. This allows flexibility in directing logs to various outputs such as:
    *   Console (during development).
    *   Log files (for persistent storage).
    *   `journald` (on Linux systems for system-wide log management).
3.  **Structured Logging Fields:** We will define a common set of structured logging fields to ensure consistency and facilitate easier parsing, querying, and analysis of logs. Standard fields should include:
    *   `timestamp`: ISO 8601 format.
    *   `level`: (ERROR, WARN, INFO, DEBUG, TRACE).
    *   `target`: The module path (e.g., `novade::compositor::handlers`).
    *   `message`: The main log message.
    *   Potentially `event_id` or `span_id` for correlating events.
    *   Component-specific fields where necessary (e.g., `window_id`, `output_name`).
4.  **Log Levels:** Clear guidelines for using log levels will be established:
    *   **ERROR:** Critical errors that prevent normal operation or lead to instability (e.g., unrecoverable backend failures, critical GTK errors).
    *   **WARN:** Unexpected situations or potential issues that do not immediately halt operation but should be investigated (e.g., configuration fallbacks, non-critical backend warnings).
    *   **INFO:** High-level information about major lifecycle events or user-driven actions (e.g., compositor startup, shell initialization, new window opened).
    *   **DEBUG:** Detailed information useful for developers to diagnose specific component behavior (e.g., event handling details, state changes).
    *   **TRACE:** Extremely verbose logging, typically for fine-grained debugging of specific functions or event flows. Should be disabled by default in release builds.
5.  **Panic Integration:** Panics (uncaught errors) will be hooked into the logging system to ensure that critical failure information, including stack traces, is captured via `tracing` before the application terminates.
6.  **Performance:** While `tracing` is highly performant, care will be taken to avoid excessive logging in hot paths, especially at `INFO` and higher levels. `DEBUG` and `TRACE` levels will be compile-time or runtime configurable to minimize overhead in production builds.

**Consequences:**

*   **Pros:**
    *   `tracing` is a modern, highly performant, and extensible framework.
    *   Provides structured logging out-of-the-box, improving observability and making log analysis easier.
    *   Widely adopted in the Rust ecosystem, with good community support and integration with other libraries.
    *   Supports asynchronous contexts well, crucial for Wayland compositors and UI frameworks.
    *   `tracing-subscriber` offers flexible configuration for various backends.
*   **Cons:**
    *   Requires consistent adoption and adherence to structured logging practices across all modules and by all developers.
    *   Might have a slight learning curve for developers unfamiliar with `tracing` compared to the simpler `log` crate.
    *   Initial setup and configuration can be more involved than simpler logging solutions.
*   **Mitigation:**
    *   Provide clear documentation, usage examples, and helper macros/functions for common logging patterns within NovaDE.
    *   Conduct brief training or provide resources for developers to get acquainted with `tracing`.
    *   Establish a default `tracing-subscriber` setup that works well for development and can be easily adapted for production.

**Alternatives Considered:**

*   **`log` crate:**
    *   **Pros:** Simpler API, very easy to integrate.
    *   **Cons:** Lacks built-in support for structured logging and spans, which are highly beneficial for a complex system like a DE. `tracing` is generally considered its successor for applications requiring more advanced features. Performance is good but `tracing` is often better in high-throughput scenarios.
*   **Custom Solution:**
    *   **Pros:** Tailored exactly to our needs.
    *   **Cons:** Significant development and maintenance overhead. Difficult to match the robustness, performance, and feature set of mature libraries like `tracing`. Likely to reinvent the wheel poorly.
*   **`env_logger` with `log`:**
    *   **Pros:** Simple environment variable-based configuration.
    *   **Cons:** Still relies on the `log` crate's limitations regarding structured logging and advanced features. Less flexible than `tracing-subscriber`.

This ADR provides the foundational logging strategy. Specific formatting details, subscriber configurations for different build profiles (debug, release), and log rotation policies will be detailed in subsequent documentation or implementation-specific ADRs if necessary.
