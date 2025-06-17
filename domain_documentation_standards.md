# Domain Layer Documentation Standards

This document outlines the comprehensive documentation standards for the domain layer, aligning with the ArchitectCodeGen principles of clarity, precision, and maintainability. Effective documentation is crucial for understanding, using, and evolving the domain model.

## 1. Inline Code Documentation (Rustdoc)

Inline documentation using `rustdoc` is the primary way to document code. It lives with the code, making it more likely to be kept up-to-date.

### Public API Documentation

All `pub` items (structs, enums, traits, functions, methods, modules) must have `///` documentation comments.

*   **Summary Line:**
    *   The first line should be a concise single sentence summarizing the item's purpose.
    *   *Example (`Order` struct):* `/// Represents a customer's order, acting as an aggregate root.`

*   **Details:**
    *   Optional subsequent paragraphs providing more detailed explanations, rationale, or usage notes.
    *   Explain *why* something is done, not just *what* is done if it's non-obvious.

*   **Parameters (`#[param]` or dedicated section):**
    *   For functions and methods, each parameter must be documented.
    *   Clearly state the purpose and expected properties of each parameter.
    *   *Example (for `Order::add_item`):*
        ```rust
        /// Adds an item to the order.
        ///
        /// # Parameters
        /// * `product_id`: The unique identifier of the product to add.
        /// * `quantity`: The number of units to add; must be greater than 0.
        /// * `unit_price`: The price of a single unit of the product at the time of adding.
        ```

*   **Return Values (`#[return]` or dedicated section):**
    *   Document the meaning of the return value.
    *   For `Result<T, E>`, document both the success (`Ok`) case and the error (`Err`) case.
    *   *Example (for `Order::add_item`):*
        ```rust
        /// # Returns
        /// * `Ok(())` if the item was successfully added.
        /// * `Err(OrderError)` if the item could not be added due to a business rule violation (e.g., invalid quantity, order status does not allow modification).
        ```

*   **Errors (`#[error]` or dedicated section):**
    *   Explicitly list and describe domain-specific errors that a function or method can return. Link to the error enum variants if possible (rustdoc does this automatically if types are used).
    *   *Example (for `Order::add_item`):*
        ```rust
        /// # Errors
        /// This method can return the following errors (see `OrderError` for more details):
        /// * `OrderError::InvalidQuantity` if the provided quantity is zero.
        /// * `OrderError::InvalidStateForOperation` if the order is not in a state that allows item additions (e.g., already shipped).
        /// * `OrderError::MoneyError` if there's a currency mismatch with existing items.
        ```

*   **Invariants, Pre-conditions, Post-conditions:**
    *   Clearly state any non-obvious conditions that must hold before calling (pre-conditions), what will be true after successful execution (post-conditions), or general rules the component maintains (invariants).
    *   *Example (for `Order::confirm`):*
        ```rust
        /// Confirms a pending order.
        ///
        /// # Pre-conditions
        /// * The order must currently be in `OrderStatus::Pending`.
        /// * The order must contain at least one item.
        ///
        /// # Post-conditions
        /// * The order's status will be `OrderStatus::Confirmed`.
        /// * The `updated_at` timestamp will be updated.
        ///
        /// # Errors
        /// * `OrderError::CannotConfirmEmptyOrder` if the order has no items.
        /// * `OrderError::InvalidStateForOperation` if the order is not `Pending`.
        ```

*   **Safety (`# Safety` section):**
    *   If `unsafe` code is used (rare in pure domain logic but possible), document the safety invariants that callers must uphold.

*   **Examples (`# Examples` section):**
    *   Provide runnable `rustdoc` examples for key public APIs. This demonstrates usage and ensures the example code compiles and works as expected.
    *   *Example (for `Money::new`):*
        ```rust
        /// Creates a new `Money` instance.
        ///
        /// # Examples
        /// ```
        /// use rust_decimal_macros::dec;
        /// use your_crate::domain::value_objects::{Money, MoneyError}; // Adjust path
        ///
        /// let a_hundred_usd = Money::new(dec!(100.00), "USD").unwrap();
        /// assert_eq!(a_hundred_usd.amount(), dec!(100.00));
        /// assert_eq!(a_hundred_usd.currency(), "USD");
        ///
        /// let invalid_money = Money::new(dec!(-5.00), "EUR");
        /// assert!(matches!(invalid_money, Err(MoneyError::NegativeAmount)));
        /// ```
        ```

### Module-Level Documentation

*   Use `//!` (inner doc comments) at the beginning of `mod.rs` files or the crate root (`lib.rs` for the domain crate, or `src/domain/mod.rs` if it's a top-level module).
*   **Purpose:** Explain the module's overall purpose, its responsibilities, the key components it contains, and how they interact or should be used together.
*   *Example (for `src/domain/aggregates/order/mod.rs`):*
    ```rust
    //! Defines the `Order` aggregate, its components, and associated business logic.
    //!
    //! The primary entry point is the `Order` struct, which acts as the aggregate root.
    //! It encapsulates `OrderItem`s and manages the order's lifecycle through various
    //! `OrderStatus` states. All modifications to an order must go through methods
    //! on the `Order` struct to ensure invariants are maintained.
    //!
    //! Key components:
    //! - `Order`: The aggregate root.
    //! - `OrderItem`: Represents a line item within an order.
    //! - `OrderStatus`: Enum defining the possible states of an order.
    //! - `OrderError`: Enum defining errors specific to order operations.
    ```

### Internal Comments (`//`)

*   Use `//` for clarifying complex, non-obvious, or tricky internal logic that isn't part of the public API.
*   Focus on *why* something is done in a particular way if it's not immediately clear.
*   Avoid over-commenting simple, self-explanatory code. The code itself should be the primary source of truth.
*   Mark `TODO:`, `FIXME:`, or `NOTE:` comments where appropriate for maintainability.

## 2. Generated API Documentation (`cargo doc`)

*   **Source:** Emphasize that the inline `rustdoc` comments are the single source of truth for the generated API documentation.
*   **Process:**
    *   Regularly generate the documentation using `cargo doc --open` (or `cargo doc --no-deps` for faster local builds).
    *   Review the generated output to ensure it is clear, complete, well-formatted, and user-friendly for developers consuming the domain layer.
    *   This review process helps catch missing documentation, unclear explanations, or formatting issues.

## 3. Architecture Decision Records (ADRs)

*   **Purpose:** To document significant design decisions, their context, alternatives considered, trade-offs, and consequences. ADRs provide a historical record of architectural evolution.
*   **Scope for Domain Layer:**
    *   **Aggregate Boundaries:** Justification for choosing specific aggregate boundaries (e.g., why `Order` includes `OrderItem` directly vs. `OrderItem` being a separate aggregate).
    *   **Domain Event Design:** Decisions around the granularity of domain events, how they are published or consumed, or complex event sourcing strategies.
    *   **Complex Business Logic Patterns:** Choice of specific design patterns (e.g., Strategy, Specification) for implementing complex or evolving business rules.
    *   **Error Handling Strategy:** Rationale for specific error types or patterns across the domain layer.
    *   **Deviations:** Reasons for deviating from a common or established pattern if deemed necessary for a specific context.
*   **Format:**
    *   Use a standard ADR format (e.g., Markdown files). A common template includes:
        *   **Title:** Short descriptive title.
        *   **Status:** (e.g., Proposed, Accepted, Deprecated, Superseded by ADR-XXX).
        *   **Date:** Date of last status change.
        *   **Context:** The problem, forces, or constraints driving the decision.
        *   **Decision:** The chosen solution or design.
        *   **Alternatives Considered:** Brief description of other options explored.
        *   **Consequences:** Positive and negative impacts of the decision (e.g., on complexity, performance, maintainability).
*   **Storage:**
    *   Store ADRs in a dedicated directory within the repository, typically `docs/adr/` or `adr/`.
    *   Use sequential numbering for ADR filenames (e.g., `001-order-aggregate-composition.md`).

## 4. READMEs and High-Level Overviews

*   **Domain Layer README (`src/domain/README.md` or `README.md` in domain crate root):**
    *   **Purpose:** Provide a high-level conceptual overview of the domain layer.
    *   **Contents:**
        *   Brief description of the domain itself (e.g., e-commerce, booking system).
        *   Explanation of core domain concepts and terminology.
        *   Overview of the main sub-modules (e.g., aggregates, value objects, services, repositories) and their responsibilities.
        *   How these components generally interact.
        *   Key architectural patterns or principles applied within the domain layer.
        *   Pointers to more detailed documentation (e.g., ADRs, generated API docs).
*   **Complex Module READMEs:**
    *   For particularly complex sub-modules within the domain (e.g., a pricing engine, a scheduling algorithm module), a local `README.md` within that module's directory can provide a more focused overview of its specific responsibilities, design, and components.

## 5. Visual Documentation (Conceptual)

While `ArchitectCodeGen` primarily focuses on textual output, the value of visual aids for complex systems should be acknowledged.

*   **Purpose:** To clarify complex relationships, flows, or state transitions that are hard to grasp from text alone.
*   **Usage:**
    *   Can be embedded in ADRs or READMEs.
    *   Use text-based diagramming tools like PlantUML or Mermaid, whose source can be version-controlled alongside Markdown files.
*   **Examples of Diagrams:**
    *   **Entity-Relationship Diagrams (ERDs - conceptual):** High-level relationships between key entities/aggregates.
    *   **State Machine Diagrams:** For entities with complex state transitions (e.g., `OrderStatus`).
        *   *Example:* A state diagram for `Order` showing transitions between `Pending`, `Confirmed`, `Shipped`, `Cancelled`.
    *   **Sequence Diagrams:** For illustrating interactions between domain services and aggregates for specific use cases or domain event flows.
    *   **Module Dependency Diagrams:** Conceptual overview of how major domain modules relate.

By adhering to these documentation standards, the domain layer will be more understandable, maintainable, and easier for new team members to onboard, fulfilling the quality expectations of the ArchitectCodeGen persona.
