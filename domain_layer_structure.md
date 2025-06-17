# Proposed Rust Domain Layer Structure

This document outlines a typical directory structure and module organization for the domain layer of a Rust project. This structure aims to promote clear separation of concerns, modularity, high cohesion, loose coupling, and easy navigation, aligning with established domain-driven design principles.

The domain layer can either be a dedicated crate or a top-level module (e.g., `project_root/src/domain/`) within a larger workspace. The example below assumes it's a module within `src`. If it were a separate crate, `src/domain/lib.rs` would be its crate root.

## Directory Structure and Module Explanation

```
src/
|
|-- domain/
|   |-- mod.rs                 // Re-exports public elements of the domain layer.
|   |
|   |-- entities/              // Contains the core domain entities.
|   |   |-- mod.rs             // Module declaration and re-exports for entities.
|   |   |-- user.rs            // Example: User entity.
|   |   |-- product.rs         // Example: Product entity.
|   |   |-- appointment.rs     // Example: Appointment entity.
|   |   |-- ...                // Other entities.
|   |
|   |-- value_objects/         // Contains domain value objects.
|   |   |-- mod.rs             // Module declaration and re-exports for value objects.
|   |   |-- email_address.rs   // Example: EmailAddress value object.
|   |   |-- money.rs           // Example: Money value object (with currency).
|   |   |-- time_range.rs      // Example: TimeRange value object.
|   |   |-- ...                // Other value objects.
|   |
|   |-- aggregates/            // Optional: For explicit aggregate roots if they are complex.
|   |   |                      // Often, entities can also serve as aggregate roots.
|   |   |-- mod.rs             // Module declaration and re-exports for aggregates.
|   |   |-- order.rs           // Example: Order aggregate (might include OrderItem entities internally).
|   |   |-- shopping_cart.rs   // Example: ShoppingCart aggregate.
|   |   |-- ...                // Other aggregates.
|   |
|   |-- services/              // Contains domain services.
|   |   |-- mod.rs             // Module declaration and re-exports for domain services.
|   |   |-- transfer_service.rs // Example: Service for transferring money between accounts.
|   |   |-- inventory_service.rs// Example: Service for checking product availability.
|   |   |-- ...                // Other domain services.
|   |
|   |-- repositories/          // Defines traits (interfaces) for data persistence.
|   |   |-- mod.rs             // Module declaration and re-exports for repository traits.
|   |   |-- user_repository.rs // Example: Trait for User persistence.
|   |   |-- order_repository.rs// Example: Trait for Order persistence.
|   |   |-- ...                // Other repository interfaces.
|   |
|   |-- events/                // Defines domain events.
|   |   |-- mod.rs             // Module declaration and re-exports for domain events.
|   |   |-- user_registered.rs // Example: Event for when a user has registered.
|   |   |-- order_confirmed.rs // Example: Event for when an order is confirmed.
|   |   |-- ...                // Other domain events.
|   |
|   |-- error.rs               // Defines common domain error types or a module for errors.
|   |                          // Can also be `errors/mod.rs` with specific error files.
|   |
|   |-- policies/              // Optional: For explicit domain policies or specifications.
|   |   |-- mod.rs             // Module declaration and re-exports for policies.
|   |   |-- discount_policy.rs // Example: Policy for applying discounts.
|   |   |-- ...                // Other policies.
|
|-- application/               // Application layer (use cases, commands, queries).
|   |-- ...
|
|-- infrastructure/            // Infrastructure layer (database implementations, external services).
|   |-- ...
|
|-- main.rs                    // Main application entry point or lib.rs if this is a library crate.
```

## Explanation of Key Directories/Modules:

*   **`domain/mod.rs`**:
    *   Serves as the public API of the domain layer.
    *   It re-exports essential types, traits, and functions from the submodules that need to be accessible to other layers (like the application layer).
    *   Helps in controlling the visibility and public interface of the domain.

*   **`domain/entities/`**:
    *   **Purpose**: Houses the definitions of domain entities. Entities are core objects with a distinct identity that persists over time and through state changes. They encapsulate attributes and behaviors specific to them.
    *   **Contents**: Each entity typically gets its own file (e.g., `user.rs`, `product.rs`).
    *   **`mod.rs`**: Declares the entity modules and may re-export them for easier access within the domain layer.

*   **`domain/value_objects/`**:
    *   **Purpose**: Contains value objects. These are objects defined by their attributes, are immutable, and lack a distinct identity. They represent descriptive aspects of the domain.
    *   **Contents**: Each value object in its own file (e.g., `email_address.rs`, `money.rs`).
    *   **`mod.rs`**: Declares value object modules and may re-export them.

*   **`domain/aggregates/`**:
    *   **Purpose**: Defines aggregates, which are clusters of entities and value objects treated as a single unit. An aggregate has a root entity (the aggregate root) through which all interactions with the aggregate occur. This helps maintain consistency and invariants.
    *   **Contents**: Each aggregate root and its associated components (if not defined elsewhere) (e.g., `order.rs` which might internally manage `order_item.rs` - though `OrderItem` could also be an entity).
    *   **Note**: Simpler entities might directly act as aggregate roots without needing a separate `aggregates/` directory if the complexity doesn't warrant it. The `entities/` directory might then just contain these aggregate roots.

*   **`domain/services/`**:
    *   **Purpose**: Holds domain services. These are stateless operations or pieces of domain logic that don't naturally fit within an entity or value object. They often coordinate actions between multiple domain objects.
    *   **Contents**: Files for each domain service (e.g., `transfer_service.rs`).
    *   **`mod.rs`**: Declares and re-exports domain services.

*   **`domain/repositories/`**:
    *   **Purpose**: Defines the interfaces (Rust traits) for data persistence. These traits abstract the storage mechanism, allowing the domain layer to remain independent of specific database technologies.
    *   **Contents**: Traits for each repository (e.g., `user_repository.rs`, `order_repository.rs`).
    *   **`mod.rs`**: Declares and re-exports repository traits. The actual implementations reside in the `infrastructure` layer.

*   **`domain/events/`**:
    *   **Purpose**: Contains definitions for domain events. These represent significant occurrences within the domain that other parts of the system (or even external systems) might be interested in.
    *   **Contents**: Each domain event in its own file (e.g., `user_registered.rs`).
    *   **`mod.rs`**: Declares and re-exports domain events.

*   **`domain/error.rs` (or `domain/errors/`)**:
    *   **Purpose**: Defines custom error types that are specific to the domain layer. This promotes clear and robust error handling.
    *   **Contents**: Typically an enum (or multiple enums) representing various domain-specific failure conditions, often using `thiserror` for convenience. If many errors, it might be a submodule `errors/mod.rs` with `errors/user_error.rs`, etc.

*   **`domain/policies/`** (Optional):
    *   **Purpose**: For more complex business rules or conditions that don't fit neatly into entities or services, policies (or specifications) can be defined here. They encapsulate specific decision-making logic.
    *   **Contents**: Files for each policy or specification (e.g., `discount_policy.rs`).

## Rationale

This structure promotes:
*   **High Cohesion**: Related concepts (like all entities, or all value objects) are grouped together.
*   **Loose Coupling**: The domain layer defines interfaces (repository traits) that the infrastructure layer implements, decoupling the domain from specific persistence technologies. Domain events also facilitate loose coupling between different parts of the domain or with other layers.
*   **Clear Boundaries**: The `domain` module itself acts as a clear boundary. Its `mod.rs` file carefully exposes only what's necessary.
*   **Testability**: Individual components (entities, value objects, services) can be unit tested in isolation more easily. Repository traits allow for mock implementations during testing.
*   **Understandability**: The naming and organization make it easier for developers to find specific pieces of domain logic.

This structure is a common starting point and can be adapted based on the specific needs and complexity of the project. For very small domains, some of these submodules might be overkill, but for medium to large applications, this level of organization is beneficial.
