# ExampleEchoService D-Bus API

This document describes the D-Bus API for the `ExampleEchoService`.

**Service Name**: `org.novade.ExampleEchoService`

## Root Object Path: `/org/novade/ExampleEchoService`

This is the primary path where the `ObjectManager` for this service resides.

### Interfaces on Root Path

1.  **`org.freedesktop.DBus.ObjectManager`**
    *   **Description**: Manages all D-Bus objects exposed by the `ExampleEchoService`. Clients can use the `GetManagedObjects` method on this interface to discover all objects, their interfaces, and properties provided by this service.
    *   **Methods**:
        *   `GetManagedObjects() -> (a{oa{sa{sv}}})`
            *   Returns a dictionary where keys are object paths and values are dictionaries of interface names mapped to their properties.
    *   **Signals**:
        *   `InterfacesAdded(ObjectPath object_path, Dict<String, Dict<String, Variant>> interfaces_and_properties)`
            *   Emitted when new objects or interfaces are added under the management of this `ObjectManager`.
        *   `InterfacesRemoved(ObjectPath object_path, Array<String> interfaces)`
            *   Emitted when objects or interfaces are removed.

2.  **`org.freedesktop.DBus.Properties`**
    *   **Description**: Provides property access for the `ObjectManager` interface itself (if any properties were defined for it, though typically it has none).
    *   Standard methods: `Get`, `Set`, `GetAll`.
    *   Standard signals: `PropertiesChanged`.

3.  **`org.freedesktop.DBus.Introspectable`**
    *   **Description**: Provides an XML description of the D-Bus interfaces, methods, signals, and properties available at this path.
    *   Standard method: `Introspect() -> (String xml_data)`.

## Main Object Path: `/org/novade/ExampleEchoService/Main`

This path hosts the core functionality of the `ExampleEchoService`.

### Interfaces on Main Object Path

1.  **`org.novade.ExampleEchoService.Echo`**
    *   **Description**: The primary interface for the echo service.
    *   **Methods**:
        *   **`EchoString(String input_string) -> (String output_string)`**
            *   **Description**: Takes a string as input, prepends the current value of the `Prefix` property, and returns the combined string. This method also increments the `EchoCount` property.
            *   **Input**:
                *   `input_string` (String): The string to be echoed.
            *   **Output**:
                *   `output_string` (String): The echoed string, formatted as `"{Prefix} {input_string}"`.
    *   **Properties (Exposed via `org.freedesktop.DBus.Properties` on this same path)**:
        *   **`Prefix`** (Type: `String`, Access: Read-Write)
            *   **Description**: The string that is prepended to the input of the `EchoString` method.
            *   **Default Value**: `"Echo: "`
        *   **`EchoCount`** (Type: `Int64` (`x`), Access: Read-Only)
            *   **Description**: A counter indicating the number of times the `EchoString` method has been successfully called since the service started or the property was last reset (if applicable).
            *   **Initial Value**: `0`

2.  **`org.freedesktop.DBus.Properties`**
    *   **Description**: Manages access to the properties of the `org.novade.ExampleEchoService.Echo` interface (i.e., "Prefix" and "EchoCount").
    *   Standard methods: `Get`, `Set`, `GetAll`.
    *   Standard signals: `PropertiesChanged`.

3.  **`org.freedesktop.DBus.Introspectable`**
    *   **Description**: Provides XML introspection data for this object path.

## Sub-Objects (Managed by ObjectManager)

The service can dynamically create and expose sub-objects under the `/org/novade/ExampleEchoService/sub/` path hierarchy. These sub-objects are primarily defined by their data exposed through the `ObjectManager`.

### Path: `/org/novade/ExampleEchoService/sub/{name}`

Where `{name}` is a unique identifier for the sub-object.

### Interfaces on Sub-Object Paths (as reported by ObjectManager)

1.  **`org.novade.ExampleEchoService.SubObjectData`**
    *   **Description**: A simple interface to hold basic data for a sub-object. These sub-objects might not serve this interface directly for method calls but their properties are exposed via `ObjectManager` under this interface name.
    *   **Properties (Exposed via `org.freedesktop.DBus.Properties` as part of `ObjectManager`'s `GetManagedObjects` data)**:
        *   **`Label`** (Type: `String`, Access: Read-Only)
            *   **Description**: A descriptive label for the sub-object. The value is set when the sub-object is created by the service.

---

This API structure allows clients to:
- Discover all objects and their properties using the `ObjectManager` at the root path.
- Interact with the main echo functionality via the `Echo` interface at `/org/novade/ExampleEchoService/Main`.
- Get and set the `Prefix` property and get the `EchoCount` property for the `Echo` interface.
- Be notified of property changes on the `Echo` interface.
- Be notified of the addition or removal of any objects (including sub-objects) managed by the service.
