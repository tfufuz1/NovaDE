# Core Components of a Domain Layer

This document details the core components typically found in a domain layer, building upon the principles outlined in `domain_layer_principles.md` and the structure proposed in `domain_layer_structure.md`.

## 1. Entities

*   **Purpose:**
    *   Represent core domain objects that have a distinct, continuous identity throughout their lifecycle, regardless of changes to their attributes.
    *   Encapsulate domain logic and business rules directly related to them.
    *   Often act as Aggregate Roots.

*   **Key Characteristics (as per ArchitectCodeGen persona):**
    *   **Rust Representation:** Typically `struct`s.
    *   **Identity:** Possess a unique identifier (e.g., a UUID, database ID) that remains constant. This ID distinguishes one entity from another, even if their attributes are identical.
        *   The ID is usually `pub` or `pub(crate)` for reading, but its modification is strictly controlled, often set only at creation.
    *   **Mutability:**
        *   Entities are generally mutable as their state can change over time due to domain operations.
        *   Mutability is controlled: fields are often private, and state changes occur through public methods that enforce invariants.
        *   Example: `let mut user = UserRepository::find(id).unwrap(); user.change_email(new_email);`
    *   **Lifetime:** Their lifetime is significant and tracked; they are created, loaded, modified, and eventually might be archived or deleted.
    *   **Enforcing Invariants:**
        *   Invariants (business rules that must always be true) are enforced through methods that modify the entity's state.
        *   Constructors (`new` or factory methods) ensure an entity is created in a valid state.
        *   Methods return `Result<_, DomainError>` to signal failures in upholding invariants.
    *   **Relationship with Other Components:**
        *   May hold other Entities or Value Objects as attributes.
        *   Can be part of an Aggregate.
        *   Interacted with by Domain Services and Application Services.
        *   Persisted and retrieved via Repository Interfaces.
    *   **Visibility:**
        *   Struct fields are typically private (`private` or `pub(crate)` if within the same crate's domain module).
        *   Public methods (`pub fn`) provide controlled access and operations, forming the entity's contract.

*   **Example Snippet Idea (Conceptual):**
    ```rust
    // in domain/entities/user.rs
    pub struct User {
        id: UserId, // UserId might be a Value Object
        email: EmailAddress, // EmailAddress is a Value Object
        // other private fields
    }

    impl User {
        // Constructor ensures initial validity
        pub fn new(id: UserId, email: EmailAddress) -> Result<Self, UserError> { /* ... */ }
        // Method enforcing invariants
        pub fn change_email(&mut self, new_email: EmailAddress) -> Result<(), UserError> { /* ... */ }
        // Getter for ID (immutable view)
        pub fn id(&self) -> &UserId { &self.id }
    }
    ```

## 2. Value Objects

*   **Purpose:**
    *   Represent descriptive aspects of the domain that do not have a conceptual identity. They are defined by their attributes.
    *   Measure, quantify, or describe a thing in the domain.
    *   Ensure correctness and meaning for simple values by encapsulating validation and behavior.

*   **Key Characteristics (as per ArchitectCodeGen persona):**
    *   **Rust Representation:** Typically `struct`s, sometimes `enum`s for a fixed set of values. Often derive `PartialEq`, `Eq`, `Clone`, `Debug`.
    *   **Identity:** No conceptual identity. Two Value Objects are equal if all their attributes are equal.
    *   **Mutability:** Strictly immutable. Once created, their state cannot change. Any "modification" results in a new Value Object instance.
        *   Example: `let new_address = old_address.with_street("New Street");`
    *   **Lifetime:** Their lifetime is typically tied to the Entity or Aggregate that holds them. They are created and discarded as needed.
    *   **Enforcing Invariants:**
        *   Invariants are enforced primarily during construction. The constructor (`new` or factory method) validates all attributes and returns a `Result` if validation fails.
        *   Because they are immutable, their state cannot become invalid after creation.
    *   **Relationship with Other Components:**
        *   Composed within Entities and Aggregates as attributes.
        *   Passed as parameters to Domain Services or Entity methods.
    *   **Visibility:**
        *   Fields can be `pub` if the Value Object is simple and its construction guarantees validity.
        *   Alternatively, fields can be private with public getters if more complex internal representation is needed, but the emphasis is on immutability and equality.

*   **Example Snippet Idea (Conceptual):**
    ```rust
    // in domain/value_objects/email_address.rs
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct EmailAddress {
        value: String, // private field
    }

    impl EmailAddress {
        // Constructor validates the email format
        pub fnnew(email_str: String) -> Result<Self, ValidationError> {
            // validation logic...
            Ok(EmailAddress { value: email_str })
        }
        pub fnas_str(&self) -> &str { &self.value }
    }
    ```

## 3. Aggregates

*   **Purpose:**
    *   A cluster of associated Entities and Value Objects treated as a single unit of consistency.
    *   Define a consistency boundary for transactions and invariants.
    *   Each Aggregate has a root Entity, known as the Aggregate Root.

*   **Key Characteristics (as per ArchitectCodeGen persona):**
    *   **Rust Representation:** The Aggregate Root is an Entity (`struct`). The aggregate itself is a conceptual boundary rather than a distinct Rust type, though the root entity's module might encapsulate related private types.
    *   **Identity:** The identity of the Aggregate is the identity of its Aggregate Root Entity.
    *   **Mutability:** The Aggregate Root is mutable, but all modifications to any part of the aggregate must go through methods on the Aggregate Root. Internal entities might be mutable only via the root.
    *   **Lifetime:** Tied to the lifetime of the Aggregate Root.
    *   **Enforcing Invariants:**
        *   The Aggregate Root is responsible for enforcing invariants that span multiple objects within the aggregate.
        *   External objects can only hold references to the Aggregate Root, not to internal members directly (if those members are not themselves aggregate roots of other aggregates).
        *   All operations on the aggregate are routed through the Aggregate Root, which ensures consistency.
    *   **Relationship with Other Components:**
        *   Composed of a root Entity and potentially other Entities and Value Objects.
        *   Referenced by its ID. Other aggregates are referred to by their ID only, not direct object references.
        *   Handled by Repositories (typically one repository per aggregate type).
    *   **Visibility:**
        *   The Aggregate Root Entity is public.
        *   Internal components of the aggregate might be `pub(crate)` or private to the aggregate's module, accessible only via the Aggregate Root's methods.

*   **Example Snippet Idea (Conceptual):**
    ```rust
    // in domain/aggregates/order.rs (Order is the Aggregate Root)
    pub struct Order {
        id: OrderId,
        customer_id: CustomerId,
        items: Vec<OrderItem>, // OrderItem might be a private struct or an entity managed by Order
        status: OrderStatus,   // OrderStatus is likely an enum
    }

    // OrderItem could be defined in the same module and not publicly exported
    struct OrderItem { /* ... */ }

    impl Order {
        pub fn new(/*...*/) -> Result<Self, OrderError> { /* ... */ }
        pub fn add_item(&mut self, product_id: ProductId, quantity: u32, price: Money) -> Result<(), OrderError> {
            // Check invariants, e.g., max items, product availability (via a domain service if needed)
            // Create and add OrderItem
            // ...
        }
        pub fn confirm(&mut self) -> Result<(), OrderError> { /* Ensure items exist, payment processed etc. */ }
    }
    ```

## 4. Domain Services

*   **Purpose:**
    *   Encapsulate domain logic that doesn't naturally belong to any single Entity or Value Object.
    *   Often coordinate activities or calculations involving multiple domain objects.
    *   Perform operations that are stateless from the perspective of the service itself (though they may use or modify domain objects).

*   **Key Characteristics (as per ArchitectCodeGen persona):**
    *   **Rust Representation:** Typically `struct`s with methods, or sometimes just free-standing functions if no state/dependencies are needed for the service itself (though struct-based services are easier to mock/test if they have dependencies like repository interfaces).
    *   **Identity:** No identity. They are stateless.
    *   **Mutability:** Services themselves are generally stateless and thus immutable. The methods operate on instances of Entities or Value Objects.
    *   **Lifetime:** Usually transient; they are instantiated or invoked for a specific operation and then discarded.
    *   **Enforcing Invariants:** They don't typically enforce invariants on individual entities (that's the entity's job) but might enforce broader business rules or orchestrate actions that maintain consistency across aggregates.
    *   **Relationship with Other Components:**
        *   Operate on Entities and Value Objects.
        *   May use Repository Interfaces to fetch or persist domain objects.
        *   Should not hold state related to a specific use case; state resides in Entities.
    *   **Visibility:**
        *   Service structs and their methods are `pub` or `pub(crate)` as needed by the application layer or other domain components.

*   **Example Snippet Idea (Conceptual):**
    ```rust
    // in domain/services/transfer_service.rs
    pub struct TransferService {
        // Could hold references to repository traits or other services if needed for its operations,
        // making it more testable.
        // user_repository: Arc<dyn UserRepository>,
    }

    impl TransferService {
        // Operation that involves multiple entities (e.g., two User accounts)
        pub fn transfer_funds(
            &self,
            from_account: &mut Account, // Account is an Entity
            to_account: &mut Account,
            amount: Money // Money is a Value Object
        ) -> Result<(), TransferError> {
            // Logic for transferring funds, checking balances, applying fees, etc.
            // This involves methods on the Account entity itself.
            from_account.withdraw(amount)?;
            to_account.deposit(amount)?;
            // Potentially create a domain event
            Ok(())
        }
    }
    ```

## 5. Repository Interfaces (Traits)

*   **Purpose:**
    *   Define contracts (interfaces) for data persistence and retrieval mechanisms for Aggregates.
    *   Abstract the underlying data storage technology from the domain layer, enabling loose coupling (Dependency Inversion Principle).
    *   Allow the domain logic to be persistence-ignorant.

*   **Key Characteristics (as per ArchitectCodeGen persona):**
    *   **Rust Representation:** `trait`s.
    *   **Identity:** Not applicable. They are contracts.
    *   **Mutability:** Methods might take `&self` or `&mut self` depending on whether the repository implementation needs to cache or manage internal state (though often `&self` is sufficient for the trait definition). The objects they return (Entities) are mutable.
    *   **Lifetime:** Trait methods often use generic lifetimes (e.g., `'a`) or deal with owned objects.
    *   **Enforcing Invariants:** Repositories themselves don't enforce domain invariants; that's the role of Aggregates/Entities. They ensure data is persisted and retrieved correctly.
    *   **Relationship with Other Components:**
        *   Used by Domain Services and Application Services to fetch and save Aggregates.
        *   Implemented by concrete types in the Infrastructure layer.
        *   Return and accept Entities/Aggregates.
    *   **Visibility:**
        *   Traits and their methods are `pub` so they can be implemented by the infrastructure layer and used by the application layer or domain services.

*   **Example Snippet Idea (Conceptual):**
    ```rust
    // in domain/repositories/user_repository.rs
    // Assume User is an Aggregate Root and UserId is its ID (a Value Object)
    pub trait UserRepository {
        fn find_by_id(&self, id: &UserId) -> Result<Option<User>, RepositoryError>;
        fn save(&self, user: &User) -> Result<(), RepositoryError>;
        fn delete(&self, user: &User) -> Result<(), RepositoryError>;
        // Other methods like find_by_email, etc.
    }
    ```

## 6. Domain Events

*   **Purpose:**
    *   Represent significant occurrences or facts that have happened within the domain.
    *   Enable decoupling of different parts of the domain or communication with other layers/systems in response to domain changes.
    *   Capture a historical record of state changes or important business moments.

*   **Key Characteristics (as per ArchitectCodeGen persona):**
    *   **Rust Representation:** Typically `struct`s, representing the data associated with the event. Sometimes an `enum` if there's a hierarchy of related events.
    *   **Identity:** Can have an event ID for tracking/auditing, and a timestamp. The primary "identity" is the fact that it occurred.
    *   **Mutability:** Immutable. Once an event has occurred and is created, it represents a fact in the past and should not change.
    *   **Lifetime:** Created when something notable happens. Their lifetime depends on how they are processed (e.g., immediately handled, stored in a queue, persisted for audit).
    *   **Enforcing Invariants:** Not applicable in the same way as entities. They are data carriers representing a past state change or occurrence. Their structure should be valid.
    *   **Relationship with Other Components:**
        *   Created by Entities (especially Aggregate Roots) or Domain Services when a significant state change or business moment occurs.
        *   Can be published and subscribed to by other domain components (e.g., other aggregates reacting to an event) or by the Application/Infrastructure layers (e.g., for sending notifications, updating read models).
    *   **Visibility:**
        *   Event structs and their fields are typically `pub` as they are data carriers intended for consumption by other parts of the system.

*   **Example Snippet Idea (Conceptual):**
    ```rust
    // in domain/events/user_registered_event.rs
    use chrono::{DateTime, Utc};

    #[derive(Debug, Clone)] // Often serializable if sent over a bus
    pub struct UserRegisteredEvent {
        pub event_id: Uuid, // Unique ID for the event instance
        pub occurred_on: DateTime<Utc>,
        pub user_id: UserId, // ID of the user that was registered
        pub email: EmailAddress,
    }

    impl UserRegisteredEvent {
        pub fn new(user_id: UserId, email: EmailAddress) -> Self {
            Self {
                event_id: Uuid::new_v4(),
                occurred_on: Utc::now(),
                user_id,
                email,
            }
        }
    }
    ```
This structure provides a solid foundation for building a robust and maintainable domain layer in Rust.
