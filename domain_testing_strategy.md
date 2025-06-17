# Domain Layer Testing Strategy

This document outlines a comprehensive testing strategy for domain layer components, adhering to the "ArchitectCodeGen" persona guidelines. The goal is to ensure a robust, reliable, and well-documented domain layer. This strategy references examples from the `order_aggregate_example.md` for context.

## 1. General Principles

*   **Test-Driven Development (TDD) / Behavior-Driven Development (BDD) Focus:**
    *   **TDD:** Write tests *before* writing the domain logic. This helps in clearly defining the expected behavior and ensures that code is written to meet specific requirements.
        *   *Example:* Before implementing `Order::add_item`, write tests for adding an item successfully, adding an item with invalid quantity, and adding an item to an already shipped order.
    *   **BDD:** For more complex interactions or user-facing features driven by domain logic, BDD scenarios (e.g., using Gherkin syntax, potentially with tools like `cucumber-rust`) can describe behavior in a domain-specific language. These scenarios would then be implemented via tests that drive the domain logic.
*   **High Code Coverage:**
    *   Strive for high (but pragmatic) code coverage for the domain layer, as it contains the core business logic. Coverage tools (e.g., `grcov`, `tarpaulin`) should be used to identify untested paths.
    *   The focus is on *logical path coverage* and *business rule coverage*, not just line coverage.
*   **Tests as Documentation:**
    *   Well-written tests serve as executable documentation, clearly illustrating how domain components are intended to be used and how they behave under various conditions.
    *   Test names should be descriptive, reflecting the scenario being tested (e.g., `test_order_add_item_fails_if_order_shipped()`).

## 2. Unit Testing

Unit tests focus on testing individual components (Value Objects, Entities, Aggregates, Domain Services) in isolation.

### Value Objects

*   **Purpose:** Verify correctness, immutability, and validation logic.
*   **Strategy:**
    *   **Creation (Valid Inputs):** Test successful creation with valid data.
        *   *Example (`Money` value object):* `Money::new(dec!(100.00), "USD")` should succeed.
    *   **Creation (Invalid Inputs):** Test that constructors or factory methods return appropriate errors for invalid data.
        *   *Example (`Money` value object):* `Money::new(dec!(-10.00), "USD")` should return `Err(MoneyError::NegativeAmount)`. `Money::new(dec!(10.00), "US")` should return `Err(MoneyError::InvalidCurrencyFormat)`.
    *   **Equality:** Verify `PartialEq` and `Eq` implementations (e.g., two `Money` objects with the same amount and currency are equal).
    *   **Methods:** Test any methods defined on the Value Object.
        *   *Example (`Money` value object):* Test `add`, `sub` operations, including currency mismatch errors.
    *   **Immutability:** While Rust's type system helps, ensure that operations that "modify" a Value Object actually return a new instance, leaving the original unchanged.

### Entities / Aggregates

*   **Purpose:** Verify business logic, state transitions, invariant enforcement, and internal consistency.
*   **Strategy:**
    *   **Constructors/Factory Methods:**
        *   Test that entities/aggregates are created in a valid initial state as per domain rules.
        *   *Example (`Order` aggregate):* `Order::new(customer_id)` should result in an order with `Pending` status, an empty item list, and correct timestamps.
    *   **State Transition Methods:**
        *   For each method that modifies state (e.g., `Order::add_item`, `Order::confirm`, `Order::cancel`):
            *   Verify that the state changes correctly upon successful execution.
            *   Verify that all specified invariants are upheld. If an invariant is violated, the correct domain error should be returned, and the state should remain consistent (ideally unchanged or rolled back).
            *   Test behavior at boundary conditions (e.g., adding the maximum allowed items, confirming an empty order).
        *   *Example (`Order::confirm`):*
            *   Test that a `Pending` order with items transitions to `Confirmed`.
            *   Test that trying to confirm an empty `Pending` order returns `Err(OrderError::CannotConfirmEmptyOrder)` and status remains `Pending`.
            *   Test that trying to confirm an already `Shipped` order returns `Err(OrderError::InvalidStateForOperation)` and status remains `Shipped`.
    *   **Business Logic & Calculations:**
        *   Test any internal calculations or logic.
        *   *Example (`Order` aggregate):* Test `total_amount()` with no items, one item, multiple items, and ensure it handles currency correctly (e.g., summing items of the same currency, erroring or handling mixed currencies based on rules).
    *   **Isolation:**
        *   Entities and Aggregates should ideally be testable without mocking external dependencies like databases. Their dependencies are typically other domain objects (Value Objects, other Entities within the same aggregate) or data passed as arguments.
        *   If an entity method *must* call out to a domain service (less common for pure DDD entities, but possible), that service interface could be a parameter to the method, allowing a mock to be passed in during tests.

### Domain Services

*   **Purpose:** Test stateless domain logic that coordinates actions between multiple domain objects.
*   **Strategy:**
    *   **Operations with Various Inputs:** Test the service's methods with different valid and invalid inputs to ensure correct behavior and error handling.
    *   **Mocking Dependencies:**
        *   If a domain service depends on repository traits or other domain services, these dependencies should be mocked during unit testing.
        *   Use libraries like `mockall` or hand-rolled test doubles.
        *   *Example:* If a `PaymentProcessingService` takes an `OrderRepository` and an `ExternalPaymentGateway` trait, mock both to test the service's orchestration logic without actual DB calls or network requests.
        *   Verify that the service interacts correctly with its mocks (e.g., calls `order_repository.save()` with the updated order state).

## 3. Testing Repository Interfaces (Domain Perspective)

*   **Purpose:** While the domain layer only defines repository *traits* (e.g., `OrderRepository`), tests for components using these traits (like Domain Services or Application Services) will indirectly test the *contract* of these interfaces.
*   **Strategy:**
    *   In unit tests for services that use repository traits:
        *   Provide mock implementations of the repository trait.
        *   Configure mocks to return specific data or errors to simulate different scenarios (e.g., repository finds an order, repository doesn't find an order, repository fails to save).
        *   Assert that the service calls the repository methods as expected (e.g., `save` is called with the correct `Order` object after a successful operation).
    *   Actual implementation testing of repositories (e.g., against a test database) is typically an *integration test* and falls outside the scope of pure domain layer unit testing.

## 4. Property-Based Testing

*   **Purpose:** Test that certain properties of the code hold true for a large range of automatically generated inputs, which can uncover edge cases missed by example-based testing.
*   **Strategy:**
    *   **Identify Candidates:**
        *   **Value Objects:**
            *   *Example (`Money`):* For any two `Money` objects `m1`, `m2` of the same currency, `(m1 + m2).unwrap().amount() == m1.amount() + m2.amount()`. For any valid decimal `d` and currency `c`, `Money::new(d, c).unwrap().amount() == d`.
            *   Test that parsing and then printing an ID results in the original string (for `OrderId`, `CustomerId`, etc., if they implement `FromStr` and `Display`).
        *   **Algorithms in Domain Services or Entities:**
            *   *Example (`Order::total_amount`):* For any list of `OrderItem`s with positive quantities and prices of the same currency, the `total_amount` should be non-negative and equal to the sum of `quantity * unit_price` for each item.
        *   **State Transition Invariants:**
            *   *Example (`Order`):* After any sequence of valid `add_item` calls on a new order, the `order.status()` remains `Pending`. If `confirm()` is called successfully, `status()` is `Confirmed`.
            *   Idempotency: e.g., calling `order.confirm()` multiple times on an already confirmed order might always return success or a specific "already confirmed" error, but the state remains `Confirmed`.
    *   **Tools:** Utilize Rust crates like `proptest` or `quickcheck`.

## 5. Error Handling Tests

*   **Purpose:** Ensure robust and predictable error handling.
*   **Strategy:**
    *   For every operation that returns a `Result<T, E>`:
        *   **Correct Error Type:** Verify that under specific failure conditions, the exact expected error variant is returned.
            *   *Example (`Order::add_item`):* Adding an item with quantity 0 returns `Err(OrderError::InvalidQuantity)`. Adding to a shipped order returns `Err(OrderError::InvalidStateForOperation)`.
        *   **State Consistency:** Ensure that if an operation fails midway, the state of the entity/aggregate remains consistent and valid. For instance, if adding an item to an order involves multiple checks and fails one, no partial changes should persist in the order object. The `Order` methods from the example are designed to validate first, then apply changes, which helps.
        *   Test that all error variants defined in custom error enums (e.g., `OrderError`) can actually be produced by the domain logic.

## 6. Achieving High Coverage

*   **Logical Paths:** Focus on testing all distinct logical paths through the code, including all branches of `if`/`else`/`match` statements.
*   **Business Rules & Edge Cases:** Explicitly create test cases for each business rule and known edge case. This is more important than raw line coverage.
*   **Coverage Tools as a Guide:** Use tools like `grcov` or `tarpaulin` to identify areas of code not exercised by tests. However, don't aim for 100% line coverage if it means writing trivial tests for simple getters/setters without logic or sacrificing test clarity. The domain layer, due to its criticality, warrants a higher coverage goal than perhaps UI or infrastructure code.

## 7. Test Organization

*   **Co-location (Recommended for Unit Tests):**
    *   In Rust, it's idiomatic to place unit tests in a `#[cfg(test)]` annotated module within the same file as the code being tested (e.g., `src/domain/aggregates/order.rs` would contain a `mod tests { ... }` block).
    *   This allows tests to access private helpers or internals if absolutely necessary (though testing through the public interface is preferred).
    *   Example:
        ```rust
        // In src/domain/aggregates/order.rs
        // ... order struct and impl ...

        #[cfg(test)]
        mod tests {
            use super::*; // Imports Order, OrderStatus, OrderError, etc.
            use super::super::super::value_objects::{CustomerId, ProductId, Money}; // Adjust path
            use rust_decimal_macros::dec;

            #[test]
            fn test_new_order_is_pending() {
                let customer_id = CustomerId::new();
                let order = Order::new(customer_id);
                assert_eq!(order.status(), OrderStatus::Pending);
                assert!(order.items().is_empty());
            }

            #[test]
            fn test_add_item_to_pending_order_succeeds() {
                // ... setup ...
            }

            // ... more tests for Order ...
        }
        ```
*   **Integration-style `tests` directory:** While less common for pure domain *unit* tests, if a test requires more complex setup or spans multiple domain modules in a way that feels more like integration, it could live in the `tests/` directory at the crate root. However, for the domain layer, keeping tests close to the code is generally preferred for clarity and ease of maintenance.

By implementing this strategy, the domain layer will be thoroughly tested, ensuring its correctness, robustness, and maintainability, aligning with the quality standards of the "ArchitectCodeGen" persona.
