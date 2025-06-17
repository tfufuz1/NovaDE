# Order Aggregate: Concrete Rust Example

This document provides a concrete Rust example of an `Order` aggregate, illustrating the principles and component definitions established in `domain_layer_principles.md`, `domain_layer_structure.md`, and `domain_core_components.md`.

The code is presented as if it were structured across multiple files within the domain layer.

## 1. Value Objects

These would typically reside in `src/domain/value_objects/` in their respective files (e.g., `order_id.rs`, `money.rs`). For brevity, they are grouped here.

```rust
// src/domain/value_objects/ids.rs
use std::fmt;
use uuid::Uuid;
use serde::{Serialize, Deserialize}; // Assuming usage of Serde

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrderId(Uuid);

impl OrderId {
    pub fnnew() -> Self {
        OrderId(Uuid::new_v4())
    }

    pub fnparse_str(s: &str) -> Result<Self, uuid::Error> {
        Ok(OrderId(Uuid::parse_str(s)?))
    }

    pub fnas_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for OrderId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for OrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CustomerId(Uuid);

impl CustomerId {
    pub fnnew() -> Self {
        CustomerId(Uuid::new_v4())
    }
     pub fnparse_str(s: &str) -> Result<Self, uuid::Error> {
        Ok(CustomerId(Uuid::parse_str(s)?))
    }

    pub fnas_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for CustomerId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CustomerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProductId(Uuid);

impl ProductId {
    pub fnnew() -> Self {
        ProductId(Uuid::new_v4())
    }
    pub fnparse_str(s: &str) -> Result<Self, uuid::Error> {
        Ok(ProductId(Uuid::parse_str(s)?))
    }
    pub fnas_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for ProductId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ProductId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

```rust
// src/domain/value_objects/money.rs
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Serialize, Deserialize};
use std::ops::{Add, Sub, Mul};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Money {
    amount: Decimal,
    currency: &'static str, // Using &'static str for simplicity, could be an enum
}

impl Money {
    pub fnnew(amount: Decimal, currency: &'static str) -> Result<Self, MoneyError> {
        if currency.len() != 3 {
            return Err(MoneyError::InvalidCurrencyFormat);
        }
        if amount < dec!(0) {
            return Err(MoneyError::NegativeAmount);
        }
        // Potentially more validation, e.g. scale for the currency
        Ok(Money { amount, currency })
    }

    pub fnamount(&self) -> Decimal {
        self.amount
    }

    pub fncurrency(&self) -> &'static str {
        self.currency
    }

    pub fnzero(currency: &'static str) -> Self {
        Money { amount: dec!(0), currency }
    }
}

impl Add for Money {
    type Output = Result<Self, MoneyError>;
    fn add(self, rhs: Self) -> Self::Output {
        if self.currency != rhs.currency {
            return Err(MoneyError::CurrencyMismatch);
        }
        Ok(Money { amount: self.amount + rhs.amount, currency: self.currency })
    }
}

impl Sub for Money {
    type Output = Result<Self, MoneyError>;
    fn sub(self, rhs: Self) -> Self::Output {
        if self.currency != rhs.currency {
            return Err(MoneyError::CurrencyMismatch);
        }
        if self.amount < rhs.amount {
            return Err(MoneyError::InsufficientFunds);
        }
        Ok(Money { amount: self.amount - rhs.amount, currency: self.currency })
    }
}

impl Mul<Decimal> for Money {
    type Output = Self;
    fn mul(self, rhs: Decimal) -> Self::Output {
        Money { amount: self.amount * rhs, currency: self.currency }
    }
}


impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount, self.currency)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MoneyError {
    #[error("Currency codes must be 3 characters long.")]
    InvalidCurrencyFormat,
    #[error("Money amount cannot be negative.")]
    NegativeAmount,
    #[error("Cannot perform operation on monies with different currencies.")]
    CurrencyMismatch,
    #[error("Insufficient funds for subtraction.")]
    InsufficientFunds,
}
```

## 2. Order Aggregate Components

These would reside in `src/domain/aggregates/order.rs` or a dedicated `src/domain/aggregates/order/` module.

### 2.1. `OrderError` Enum

```rust
// src/domain/aggregates/order/error.rs (or part of order.rs)
use thiserror::Error;
use super::value_objects::MoneyError; // Assuming MoneyError is in a parent value_objects module

#[derive(Error, Debug)]
pub enum OrderError {
    #[error("Order cannot be empty to confirm.")]
    CannotConfirmEmptyOrder,
    #[error("Order item quantity must be positive.")]
    InvalidQuantity,
    #[error("Operation '{0}' not allowed in current order state '{1}'.")]
    InvalidStateForOperation(String, String), // operation, current_status
    #[error("Product with ID '{0}' not found in order items.")]
    ItemNotFound(String), // ProductId as String
    #[error("Order is already paid and cannot be modified.")]
    AlreadyPaid,
    #[error("Order has no items.")]
    IsEmpty,
    #[error("Order cancellation reason cannot be empty.")]
    CancellationReasonEmpty,
    #[error("Value object error: {0}")]
    ValueError(String), // Generic for internal value object errors
    #[error("Money operation error: {0}")]
    MoneyError(#[from] MoneyError),
}
```

### 2.2. `OrderStatus` Enum

```rust
// src/domain/aggregates/order/status.rs (or part of order.rs)
use serde::{Serialize, Deserialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
    Delivered,
    Cancelled,
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderStatus::Pending => write!(f, "Pending"),
            OrderStatus::Confirmed => write!(f, "Confirmed"),
            OrderStatus::Shipped => write!(f, "Shipped"),
            OrderStatus::Delivered => write!(f, "Delivered"),
            OrderStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}
```

### 2.3. `OrderItem` Struct

This struct is internal to the `Order` aggregate, so it might not be `pub`.

```rust
// src/domain/aggregates/order/item.rs (or part of order.rs)
use rust_decimal::Decimal;
use super::super::value_objects::{ProductId, Money}; // Adjust path as needed
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderItem {
    product_id: ProductId,
    quantity: u32,
    unit_price: Money, // Price at the time of adding to cart
}

impl OrderItem {
    pub fnnew(product_id: ProductId, quantity: u32, unit_price: Money) -> Result<Self, super::OrderError> {
        if quantity == 0 {
            return Err(super::OrderError::InvalidQuantity);
        }
        // Ensure unit_price is not negative (Money VO should handle this, but can double check)
        if unit_price.amount() < Decimal::ZERO {
             return Err(super::OrderError::ValueError("Unit price cannot be negative.".to_string()));
        }
        Ok(Self { product_id, quantity, unit_price })
    }

    pub fn product_id(&self) -> &ProductId {
        &self.product_id
    }

    pub fn quantity(&self) -> u32 {
        self.quantity
    }

    pub fn unit_price(&self) -> Money {
        self.unit_price
    }

    pub fn line_total(&self) -> Result<Money, super::OrderError> {
        Ok(self.unit_price * Decimal::from(self.quantity))
    }
}
```

### 2.4. `Order` Aggregate Root

```rust
// src/domain/aggregates/order/mod.rs (or order.rs if flat structure)
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};

// Assuming these are correctly pathed from the perspective of order.rs
use super::super::value_objects::{OrderId, CustomerId, ProductId, Money, MoneyError};
// If OrderItem, OrderStatus, OrderError are in separate files in an 'order' module:
pub use self::error::OrderError;
pub use self::item::OrderItem;
pub use self::status::OrderStatus;

// If they are in the same file, these imports are not needed.
// For this example, assume they are separate files within an 'order' module.
mod error;
mod item;
mod status;

#[derive(Debug, Clone, Serialize, Deserialize)] // Clone for repository find_by_id, Serialize for persistence
pub struct Order {
    id: OrderId,
    customer_id: CustomerId,
    items: Vec<OrderItem>,
    status: OrderStatus,
    // cancellation_reason: Option<String>, // Could be added for Cancelled status
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Order {
    pub fnnew(customer_id: CustomerId) -> Self {
        // Initial state is Pending, no items.
        // Invariants: ID is generated, status is Pending, timestamps are set.
        Order {
            id: OrderId::new(),
            customer_id,
            items: Vec::new(),
            status: OrderStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // --- Getters ---
    pub fn id(&self) -> &OrderId { &self.id }
    pub fn customer_id(&self) -> &CustomerId { &self.customer_id }
    pub fn status(&self) -> OrderStatus { self.status }
    pub fn items(&self) -> &Vec<OrderItem> { &self.items }
    pub fn created_at(&self) -> DateTime<Utc> { self.created_at }
    pub fn updated_at(&self) -> DateTime<Utc> { self.updated_at }

    pub fn total_amount(&self) -> Result<Money, OrderError> {
        if self.items.is_empty() {
            // Depending on rules, could be an error or just zero.
            // For this example, let's assume default currency "USD" if no items.
            // A real app would need a more robust way to determine currency.
            return Ok(Money::zero("USD"));
        }
        let first_item_currency = self.items[0].unit_price().currency();
        let mut total = Money::zero(first_item_currency);
        for item in &self.items {
            total = (total + item.line_total()?)?;
        }
        Ok(total)
    }

    // --- State Transition Methods & Business Logic ---

    pub fn add_item(&mut self, product_id: ProductId, quantity: u32, unit_price: Money) -> Result<(), OrderError> {
        // Invariant: Cannot add items to an order that is not Pending or Confirmed (if re-confirmation is allowed)
        if ![OrderStatus::Pending, OrderStatus::Confirmed].contains(&self.status) {
            return Err(OrderError::InvalidStateForOperation("add_item".to_string(), self.status.to_string()));
        }
        if quantity == 0 {
            return Err(OrderError::InvalidQuantity);
        }
        // Invariant: Ensure consistent currency if items already exist
        if let Some(first_item) = self.items.first() {
            if first_item.unit_price().currency() != unit_price.currency() {
                return Err(OrderError::MoneyError(MoneyError::CurrencyMismatch));
            }
        }

        // If item already exists, one might update quantity or add as a new line.
        // For simplicity, this example adds as a new line or updates existing.
        if let Some(existing_item) = self.items.iter_mut().find(|i| i.product_id() == &product_id && i.unit_price() == unit_price) {
            existing_item.quantity += quantity;
        } else {
            let item = OrderItem::new(product_id, quantity, unit_price)?;
            self.items.push(item);
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn remove_item(&mut self, product_id: ProductId) -> Result<(), OrderError> {
        if ![OrderStatus::Pending, OrderStatus::Confirmed].contains(&self.status) {
            return Err(OrderError::InvalidStateForOperation("remove_item".to_string(), self.status.to_string()));
        }

        let initial_len = self.items.len();
        self.items.retain(|item| item.product_id() != &product_id);

        if self.items.len() == initial_len {
            return Err(OrderError::ItemNotFound(product_id.to_string()));
        }
        self.updated_at = Utc::now();
        Ok(())
    }


    pub fn confirm(&mut self) -> Result<(), OrderError> {
        // Invariant: Order must be in Pending state to be confirmed.
        if self.status != OrderStatus::Pending {
            return Err(OrderError::InvalidStateForOperation("confirm".to_string(), self.status.to_string()));
        }
        // Invariant: Order must have items to be confirmed.
        if self.items.is_empty() {
            return Err(OrderError::CannotConfirmEmptyOrder);
        }

        self.status = OrderStatus::Confirmed;
        self.updated_at = Utc::now();
        // Domain Event: OrderConfirmed could be raised here.
        Ok(())
    }

    pub fn ship(&mut self /*, tracking_number: String */) -> Result<(), OrderError> {
        if self.status != OrderStatus::Confirmed {
            return Err(OrderError::InvalidStateForOperation("ship".to_string(), self.status.to_string()));
        }
        self.status = OrderStatus::Shipped;
        self.updated_at = Utc::now();
        // Domain Event: OrderShipped could be raised here.
        Ok(())
    }

    pub fn deliver(&mut self) -> Result<(), OrderError> {
        if self.status != OrderStatus::Shipped {
            return Err(OrderError::InvalidStateForOperation("deliver".to_string(), self.status.to_string()));
        }
        self.status = OrderStatus::Delivered;
        self.updated_at = Utc::now();
        // Domain Event: OrderDelivered could be raised here.
        Ok(())
    }

    pub fn cancel(&mut self, reason: String) -> Result<(), OrderError> {
        // Invariant: Order cannot be cancelled if already Shipped or Delivered (business rule).
        if [OrderStatus::Shipped, OrderStatus::Delivered].contains(&self.status) {
            return Err(OrderError::InvalidStateForOperation("cancel".to_string(), self.status.to_string()));
        }
        if reason.trim().is_empty() {
            return Err(OrderError::CancellationReasonEmpty);
        }

        self.status = OrderStatus::Cancelled;
        // self.cancellation_reason = Some(reason);
        self.updated_at = Utc::now();
        // Domain Event: OrderCancelled could be raised here.
        Ok(())
    }
}
```

## 3. Repository Interface

This would reside in `src/domain/repositories/order_repository.rs`.

```rust
// src/domain/repositories/order_repository.rs
use super::super::aggregates::order::Order; // Adjust path to Order aggregate
use super::super::value_objects::OrderId;   // Adjust path to OrderId
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Underlying data store error: {0}")]
    StorageError(String), // More specific errors can be added
    #[error("Item not found: {0}")]
    NotFound(String),
    #[error("Failed to serialize or deserialize data: {0}")]
    SerializationError(String),
    #[error("Optimistic lock error for ID: {0}")]
    ConcurrencyError(String), // For handling concurrent updates
    #[error("Unknown repository error")]
    Unknown,
}

// Blanket implementation to allow `Box<dyn std::error::Error>` to be converted
// This is useful if the infrastructure layer uses `Box<dyn Error>`.
impl From<Box<dyn std::error::Error + Send + Sync>> for RepositoryError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        RepositoryError::StorageError(err.to_string())
    }
}


pub trait OrderRepository {
    fn save(&self, order: &Order) -> Result<(), RepositoryError>;
    fn find_by_id(&self, id: &OrderId) -> Result<Option<Order>, RepositoryError>;
    // Other methods might include:
    // fn find_by_customer_id(&self, customer_id: &CustomerId) -> Result<Vec<Order>, RepositoryError>;
    // fn next_identity(&self) -> OrderId; // If IDs are generated by the repository
}
```

This example provides a comprehensive look at an `Order` aggregate, including its internal structure, business logic, state transitions, error handling, and the definition of its repository interface. It adheres to DDD principles by encapsulating behavior and data within the aggregate root, ensuring invariants are maintained through its methods.
The Value Objects ensure data integrity at the attribute level.
This example assumes dependencies like `uuid`, `chrono`, `rust_decimal`, `serde`, and `thiserror` are available.
In a real application, you would also have domain event structs (e.g., `OrderConfirmedEvent`) created and potentially returned by the methods that cause significant state changes (or handled by an event dispatcher).
