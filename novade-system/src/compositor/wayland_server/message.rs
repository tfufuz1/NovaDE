use byteorder::{ByteOrder, NativeEndian}; // ReadBytesExt not directly used if using slices primarily
use std::os::unix::io::RawFd;
// std::io::{Cursor, Read} are not directly used by current deserializers.
// std::convert::TryInto is not explicitly used.

// Import protocol specification types and ObjectRegistry
use super::protocol_spec::{ArgumentType, ProtocolManager}; // Assuming ArgumentType is also in protocol_spec
use super::object_registry::ObjectRegistry;


#[derive(Debug, PartialEq, Clone)]
pub enum Argument {
    Int(i32),
    Uint(u32),
    Fixed(i32), // Raw fixed-point value, 1/256th of a unit
    String(String),
    Object(u32), // Existing object ID
    NewId(u32),  // ID for a new object to be created
    Array(Vec<u8>),
    Fd(RawFd),
}

impl Argument {
    /// Converts a raw fixed-point i32 to f64.
    pub fn fixed_to_f64(fixed_val: i32) -> f64 {
        fixed_val as f64 / 256.0
    }

    /// Converts an f64 to a raw fixed-point i32.
    pub fn f64_to_fixed(float_val: f64) -> i32 {
        (float_val * 256.0).round() as i32
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    pub sender_id: u32, // Object ID of the sender
    pub opcode: u16,
    pub len: u16, // Total length of the message in bytes, including header
    pub args: Vec<Argument>,
}

#[derive(Debug, PartialEq)]
pub enum MessageParseError {
    InvalidHeader(String),
    InvalidArgument(String),
    NotEnoughData(String),
    IoError(String), // For errors from Read trait, etc.
    UnsupportedOpcode(u16),
    UnsupportedInterface(u32),
}

impl From<std::io::Error> for MessageParseError {
    fn from(err: std::io::Error) -> Self {
        MessageParseError::IoError(err.to_string())
    }
}

/// Parses the 8-byte message header.
/// Returns (sender_id, opcode, length).
pub fn parse_message_header(bytes: &[u8]) -> Result<(u32, u16, u16), MessageParseError> {
    if bytes.len() < 8 {
        return Err(MessageParseError::NotEnoughData(format!(
            "Need 8 bytes for header, got {}",
            bytes.len()
        )));
    }
    let sender_id = NativeEndian::read_u32(&bytes[0..4]);
    let opcode_len_raw = NativeEndian::read_u32(&bytes[4..8]);
    let opcode = (opcode_len_raw >> 16) as u16; // Higher 16 bits
    let len = (opcode_len_raw & 0xFFFF) as u16;   // Lower 16 bits

    if len < 8 {
        return Err(MessageParseError::InvalidHeader(format!(
            "Message length {} is less than minimum header size 8",
            len
        )));
    }
    Ok((sender_id, opcode, len))
}

// Argument Deserialization Functions

fn deserialize_u32(bytes: &mut &[u8]) -> Result<u32, MessageParseError> {
    if bytes.len() < 4 {
        return Err(MessageParseError::NotEnoughData("Need 4 bytes for u32".to_string()));
    }
    let val = NativeEndian::read_u32(*bytes);
    *bytes = &bytes[4..];
    Ok(val)
}

fn deserialize_i32(bytes: &mut &[u8]) -> Result<i32, MessageParseError> {
    if bytes.len() < 4 {
        return Err(MessageParseError::NotEnoughData("Need 4 bytes for i32".to_string()));
    }
    let val = NativeEndian::read_i32(*bytes);
    *bytes = &bytes[4..];
    Ok(val)
}

#[allow(dead_code)] // Mark as used, part of the Argument enum
fn deserialize_int(bytes: &mut &[u8]) -> Result<Argument, MessageParseError> {
    deserialize_i32(bytes).map(Argument::Int)
}

fn deserialize_uint(bytes: &mut &[u8]) -> Result<Argument, MessageParseError> {
    deserialize_u32(bytes).map(Argument::Uint)
}

fn deserialize_fixed(bytes: &mut &[u8]) -> Result<Argument, MessageParseError> {
    deserialize_i32(bytes).map(Argument::Fixed)
}

fn deserialize_object_id(bytes: &mut &[u8]) -> Result<Argument, MessageParseError> {
    deserialize_u32(bytes).map(Argument::Object)
}

fn deserialize_new_id(bytes: &mut &[u8]) -> Result<Argument, MessageParseError> {
    deserialize_u32(bytes).map(Argument::NewId)
}

fn deserialize_string(bytes: &mut &[u8]) -> Result<Argument, MessageParseError> {
    if bytes.len() < 4 {
        return Err(MessageParseError::NotEnoughData("Need 4 bytes for string length".to_string()));
    }
    let len = NativeEndian::read_u32(*bytes) as usize;
    *bytes = &bytes[4..];

    if bytes.len() < len {
        return Err(MessageParseError::NotEnoughData(format!(
            "Need {} bytes for string content, got {}",
            len,
            bytes.len()
        )));
    }
    if len == 0 {
         return Err(MessageParseError::InvalidArgument("String length cannot be 0".to_string()));
    }

    // Find null terminator
    let c_str_slice = &bytes[..len-1]; // Exclude the null terminator for from_utf8
    let s = std::ffi::CStr::from_bytes_with_nul(bytes.get(..(len)).ok_or_else(|| MessageParseError::NotEnoughData("String slice out of bounds".to_string()))?)
        .map_err(|e| MessageParseError::InvalidArgument(format!("Invalid C string: {}", e)))?
        .to_str()
        .map_err(|e| MessageParseError::InvalidArgument(format!("Invalid UTF-8 string: {}", e)))?
        .to_string();

    // Advance buffer past string and padding
    let padded_len = (len + 3) & !3; // Align to 4-byte boundary
    if bytes.len() < padded_len {
        return Err(MessageParseError::NotEnoughData(format!(
            "Need {} bytes for padded string, got {}",
            padded_len,
            bytes.len()
        )));
    }
    *bytes = &bytes[padded_len..];
    Ok(Argument::String(s))
}

fn deserialize_array(bytes: &mut &[u8]) -> Result<Argument, MessageParseError> {
    if bytes.len() < 4 {
        return Err(MessageParseError::NotEnoughData("Need 4 bytes for array length".to_string()));
    }
    let len = NativeEndian::read_u32(*bytes) as usize;
    *bytes = &bytes[4..];

    if bytes.len() < len {
        return Err(MessageParseError::NotEnoughData(format!(
            "Need {} bytes for array content, got {}",
            len,
            bytes.len()
        )));
    }
    let arr_data = bytes[..len].to_vec();
    *bytes = &bytes[len..];
    // Wayland arrays are also padded to 32-bit boundary, but the content itself is not.
    // The *cursor* should be advanced by padded length if this array is followed by other args.
    // However, the Vec<u8> should contain only actual data.
    // For now, assuming array data itself is not padded, but the read cursor advances padded.
    // This needs careful handling based on how containing structures are packed.
    // For message parsing, the total message length already accounts for padding.
    // The cursor for the *next argument* needs to be aligned.
    // Let's assume for now that the buffer slice `bytes` is advanced correctly by the caller
    // or that individual argument parsers handle their own padding advancement if necessary.
    // The string deserializer does this. Array should too if it's not the last element.
    let padded_len = (len + 3) & !3;
     if bytes.len() < padded_len - len { // bytes already advanced by len
        // This check is tricky. If len is 5, padded_len is 8. We read 5. bytes is now shorter.
        // This padding is for the *start* of the next argument.
        // The current implementation correctly slices off `len` for `arr_data`.
        // The advancement of `bytes` by `padded_len` should happen *after* this function call,
        // or this function should return the number of bytes *consumed* including padding.
        // Let's return the unpadded array and let the main parser handle alignment based on arg type.
        // For now, we advance by unpadded length. String parser advances by padded length.
        // This inconsistency needs to be resolved.
        // RESOLUTION: For now, individual parsers consume their exact data. Padding is handled by the main loop.
        // String is special because its length includes null terminator and then padding.
        // Array length is explicit.
        // Let's make array also consume its padding.
         if bytes.len() < padded_len - len {
            // This means there isn't enough data left for padding
             return Err(MessageParseError::NotEnoughData(format!(
                "Need {} bytes for array padding, got {}",
                padded_len - len,
                bytes.len()
            )));
        }
        *bytes = &bytes[(padded_len - len)..];

    }


    Ok(Argument::Array(arr_data))
}

#[allow(unused_variables)] // bytes is not used for FD deserialization as FDs come from ancillary data
fn deserialize_fd(
    _bytes: &mut &[u8], // FDs are not in the byte stream, so _bytes is not used for data extraction here.
                        // It might be used by a higher-level parser to check alignment or skip space if FDs had placeholders.
    available_fds: &mut Vec<RawFd>, // Changed to a mutable Vec to consume FDs.
) -> Result<Argument, MessageParseError> {
    // File descriptors are not in the main byte stream. They are sent as ancillary data.
    // This function consumes the next available FD from the list provided.
    if let Some(fd) = available_fds.pop() { // Take from the end, assuming FDs are ordered as expected.
                                           // Or .remove(0) if they are in order of arguments.
                                           // Wayland spec implies FDs are ordered with messages.
                                           // Let's assume .remove(0) for FIFO order matching argument list.
        // Re-evaluating: if multiple FDs are for a single message, they should be consumed in order.
        // If `available_fds` is specifically for *this message*, then .remove(0) is correct.
        Ok(Argument::Fd(fd))
    } else {
        // This error means the protocol expected an FD, but none were available in the ancillary data
        // provided for this message.
        Err(MessageParseError::InvalidArgument(
            "Expected file descriptor, but none were available in the provided ancillary data.".to_string(),
        ))
    }
}


/// Parses a single Wayland message from the buffer using protocol specifications.
///
/// Args:
///     initial_buffer: Slice of bytes containing the raw message data.
///     protocol_manager: A reference to the ProtocolManager for looking up interface and request specs.
///     object_registry: A reference to the ObjectRegistry for finding the sender object's interface.
///     ancillary_fds: A mutable option containing a vector of file descriptors received with this message batch.
///                    FDs are consumed from this vector as they are parsed into arguments.
///
/// Returns:
///     A Result containing the parsed `Message` and a slice of the remaining unconsumed portion of `initial_buffer`,
///     or a `MessageParseError` if parsing fails.
pub fn parse_message<'a>(
    initial_buffer: &'a [u8],
    protocol_manager: &ProtocolManager,
    object_registry: &ObjectRegistry,
    ancillary_fds: &mut Option<Vec<RawFd>>,
) -> Result<(Message, &'a [u8]), MessageParseError> {
    let (sender_id, opcode, len) = parse_message_header(initial_buffer)?;

    if initial_buffer.len() < len as usize {
        return Err(MessageParseError::NotEnoughData(format!(
            "Message header indicates length {}, but buffer only has {} bytes",
            len,
            initial_buffer.len()
        )));
    }

    let mut message_body_buffer = &initial_buffer[8..len as usize];
    let rest_of_buffer = &initial_buffer[len as usize..];
    let mut args = Vec::new();

    // 1. Determine the interface of the sender_id
    let sender_entry = object_registry.get_entry(sender_id).ok_or_else(|| {
        // If sender_id is not found, it's a critical error, client sent message for non-existent object.
        MessageParseError::InvalidArgument(format!("Sender object ID {} not found in registry.", sender_id))
    })?;
    let interface_name = &sender_entry.interface_name;

    // 2. Get the RequestSpec for this interface and opcode
    let request_spec = protocol_manager
        .get_request_spec(interface_name, opcode)
        .ok_or_else(|| MessageParseError::UnsupportedOpcode(opcode))?; // TODO: Include interface name in error

    // 3. Iterate through ArgumentSpecs and deserialize
    // let mut total_consumed_body_bytes = 0usize; // For precise tracking if needed
    for arg_spec in &request_spec.args {
        // let initial_body_len_for_arg = message_body_buffer.len(); // For tracking consumption per arg
        let arg = match arg_spec.arg_type {
            ArgumentType::Int => deserialize_int(&mut message_body_buffer)?,
            ArgumentType::Uint => deserialize_uint(&mut message_body_buffer)?,
            ArgumentType::Fixed => deserialize_fixed(&mut message_body_buffer)?,
            ArgumentType::String => deserialize_string(&mut message_body_buffer)?,
            ArgumentType::Object => deserialize_object_id(&mut message_body_buffer)?,
            ArgumentType::NewId => deserialize_new_id(&mut message_body_buffer)?,
            ArgumentType::Array => deserialize_array(&mut message_body_buffer)?,
            ArgumentType::Fd => {
                if let Some(ref mut fds_vec) = ancillary_fds {
                    // Pass a mutable slice of the specific FD vector for this message
                    deserialize_fd(&mut message_body_buffer, fds_vec)?
                } else {
                    // This case: ancillary_fds itself is None, meaning no FD list was even available for any message in the batch.
                    return Err(MessageParseError::InvalidArgument(format!(
                        "Arg type FD expected for '{}', but no FD list (ancillary_fds was None) provided to parse_message.",
                        arg_spec.name
                    )));
                }
            }
        };
        args.push(arg);
        // total_consumed_body_bytes += initial_body_len_for_arg - message_body_buffer.len();
    }

    // 4. Sanity check: Ensure the entire message body declared by `len` was consumed
    //    by the argument deserializers, considering padding.
    //    The `message_body_buffer` should be empty if all arguments (and their padding) consumed `len - 8` bytes.
    if !message_body_buffer.is_empty() {
        eprintln!(
            "Warning: Message body not fully consumed for {} op {}. {} bytes remaining. This might be due to padding or an incomplete/mismatched protocol specification for args.",
            interface_name, opcode, message_body_buffer.len()
        );
        // Depending on strictness, this could be an error:
        // return Err(MessageParseError::InvalidArgument(format!(
        //     "Message body not fully consumed. {} bytes remaining. Interface: {}, Opcode: {}",
        //     message_body_buffer.len(), interface_name, opcode
        // )));
    }

    // The old hardcoded logic has been replaced by the dynamic parsing loop above.
    // The TODO for FD parsing logic is still relevant for the dynamic loop if an FD arg is encountered.

    Ok((
        Message {
            sender_id,
            opcode,
            len,
            args,
        },
        rest_of_buffer,
    ))
}

// --- Message Serialization ---

/// Calculates the padded length for a Wayland string or array.
fn padded_len(len: usize) -> usize {
    (len + 3) & !3
}

/// Serializes a single argument into a byte vector.
/// Does not handle FDs as they are sent via ancillary data.
fn serialize_argument(arg: &Argument, bytes: &mut Vec<u8>) -> Result<(), String> {
    match arg {
        Argument::Int(val) | Argument::Fixed(val) => bytes.extend_from_slice(&val.to_ne_bytes()),
        Argument::Uint(val) | Argument::Object(val) | Argument::NewId(val) => {
            bytes.extend_from_slice(&val.to_ne_bytes())
        }
        Argument::String(s) => {
            let string_bytes = s.as_bytes();
            let len_with_null = string_bytes.len() + 1;
            if len_with_null == 0 { // Should not happen with Rust strings typically
                return Err("String length with null cannot be zero for serialization.".to_string());
            }
            bytes.extend_from_slice(&(len_with_null as u32).to_ne_bytes());
            bytes.extend_from_slice(string_bytes);
            bytes.push(0); // Null terminator
            let current_len = bytes.len();
            let padded_total_len = padded_len(current_len - (string_bytes.len() + 1 + 4) + len_with_null); // len based on string + null
            let num_padding_bytes = padded_total_len - (string_bytes.len() + 1);


            let padding_needed = padded_len(len_with_null) - len_with_null;
            for _ in 0..padding_needed {
                bytes.push(0);
            }
        }
        Argument::Array(arr_data) => {
            bytes.extend_from_slice(&(arr_data.len() as u32).to_ne_bytes());
            bytes.extend_from_slice(arr_data);
            let padding_needed = padded_len(arr_data.len()) - arr_data.len();
            for _ in 0..padding_needed {
                bytes.push(0);
            }
        }
        Argument::Fd(_) => {
            return Err("Argument::Fd cannot be serialized into the main byte stream. FDs must be passed via ancillary data.".to_string());
        }
    }
    Ok(())
}

/// Serializes a Wayland message (header + arguments) into a byte vector.
/// FDs are not included in the byte vector; they must be handled separately.
pub fn serialize_message(
    sender_id: u32,
    opcode: u16,
    args: &[Argument],
) -> Result<Vec<u8>, String> {
    let mut arg_bytes = Vec::new();
    for arg in args {
        serialize_argument(arg, &mut arg_bytes)?;
    }

    let total_len = 8 + arg_bytes.len(); // 8 bytes for header
    if total_len > u16::MAX as usize {
        return Err(format!(
            "Serialized message length {} exceeds u16::MAX.",
            total_len
        ));
    }

    let mut message_bytes = Vec::with_capacity(total_len);
    message_bytes.extend_from_slice(&sender_id.to_ne_bytes());
    let len_opcode = ((total_len as u32) << 16) | (opcode as u32);
    message_bytes.extend_from_slice(&len_opcode.to_ne_bytes());
    message_bytes.extend_from_slice(&arg_bytes);

    Ok(message_bytes)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::io::AsRawFd;
    use std::sync::Arc;
    use super::super::protocol_spec::{self as spec, ArgumentType, ProtocolManager}; // Use alias for protocol_spec module
    use crate::compositor::wayland_server::object_registry::{ObjectRegistry, WlDisplay}; // For test setup

    // Helper to create a mock ProtocolManager and ObjectRegistry for tests
    fn setup_test_protocols_and_registry() -> (ProtocolManager, ObjectRegistry) {
        let mut pm = ProtocolManager::new();
        spec::load_core_protocols(&mut pm);

        // ObjectRegistry::new() already creates wl_display with ID 1 and interface "wl_display"
        let registry = ObjectRegistry::new();
        (pm, registry)
    }


    #[test]
    fn test_parse_message_header_valid() {
        // sender_id=1, opcode=0, len=12
        // For opcode=0, len=12: (0 << 16) | 12 = 12.
        // For opcode=1, len=16: (1 << 16) | 16 = 65536 | 16 = 65552
        let mut header_bytes = [0u8; 8];
        NativeEndian::write_u32(&mut header_bytes[0..4], 1); // sender_id = 1
        NativeEndian::write_u32(&mut header_bytes[4..8], (0u32 << 16) | 12u32); // opcode=0, len=12
        assert_eq!(parse_message_header(&header_bytes), Ok((1, 0, 12)));

        NativeEndian::write_u32(&mut header_bytes[4..8], (1u32 << 16) | 16u32); // opcode=1, len=16
        assert_eq!(parse_message_header(&header_bytes), Ok((1, 1, 16)));
    }

    #[test]
    fn test_parse_message_header_too_short() {
        let bytes = [1, 0, 0, 0, 0, 0, 12]; // 7 bytes
        let result = parse_message_header(&bytes);
        assert_matches!(result, Err(MessageParseError::NotEnoughData(_)));
    }

    #[test]
    fn test_parse_message_header_len_too_small() {
        let mut header_bytes = [0u8; 8];
        NativeEndian::write_u32(&mut header_bytes[0..4], 1);
        NativeEndian::write_u32(&mut header_bytes[4..8], (0u32 << 16) | 7u32); // len=7 (invalid)
        let result = parse_message_header(&header_bytes);
        assert_matches!(result, Err(MessageParseError::InvalidHeader(_)));
    }

    #[test]
    fn test_deserialize_u32_simple() {
        let mut bytes = [10u8, 0, 0, 0].as_slice();
        assert_eq!(deserialize_u32(&mut bytes).unwrap(), 10);
        assert!(bytes.is_empty());
    }

    #[test]
    fn test_deserialize_i32_simple() {
        let mut bytes = [0xf6u8, 0xff, 0xff, 0xff].as_slice(); // -10
        assert_eq!(deserialize_i32(&mut bytes).unwrap(), -10);
        assert!(bytes.is_empty());
    }

    #[test]
    fn test_deserialize_fixed_simple() {
        // 10.0 represented as fixed point: 10 * 256 = 2560
        // 2560 = 0x0A00
        let mut bytes = [0x00u8, 0x0A, 0, 0].as_slice();
        match deserialize_fixed(&mut bytes).unwrap() {
            Argument::Fixed(val) => assert_eq!(val, 2560),
            _ => panic!("Wrong argument type"),
        }
        assert!(bytes.is_empty());
    }

    #[test]
    fn test_deserialize_string_simple() {
        let mut data = Vec::new();
        // String "hello" (5 chars + 1 null = 6 bytes), padded to 8 bytes
        // Length prefix: 6 (u32)
        data.extend_from_slice(&6u32.to_ne_bytes());
        data.extend_from_slice(b"hello\0");
        data.extend_from_slice(&[0u8, 0u8]); // Padding
        data.extend_from_slice(b"rest"); // Some more data after string

        let mut bytes_slice = data.as_slice();
        match deserialize_string(&mut bytes_slice).unwrap() {
            Argument::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Wrong argument type"),
        }
        assert_eq!(bytes_slice, b"rest"); // Check remaining
    }

    #[test]
    fn test_deserialize_string_exact_padding() {
        let mut data = Vec::new();
        // String "hi" (2 chars + 1 null = 3 bytes), padded to 4 bytes
        data.extend_from_slice(&3u32.to_ne_bytes());
        data.extend_from_slice(b"hi\0");
        data.push(0); // Padding
        data.extend_from_slice(b"next");

        let mut bytes_slice = data.as_slice();
        match deserialize_string(&mut bytes_slice).unwrap() {
            Argument::String(s) => assert_eq!(s, "hi"),
            _ => panic!("Wrong argument type"),
        }
        assert_eq!(bytes_slice, b"next");
    }

    #[test]
    fn test_deserialize_string_no_padding_needed() {
        let mut data = Vec::new();
        // String "abc" (3 chars + 1 null = 4 bytes), no padding needed
        data.extend_from_slice(&4u32.to_ne_bytes());
        data.extend_from_slice(b"abc\0");
        data.extend_from_slice(b"next");

        let mut bytes_slice = data.as_slice();
        match deserialize_string(&mut bytes_slice).unwrap() {
            Argument::String(s) => assert_eq!(s, "abc"),
            _ => panic!("Wrong argument type"),
        }
        assert_eq!(bytes_slice, b"next");
    }


    #[test]
    fn test_deserialize_array_simple() {
        let mut data = Vec::new();
        // Array: [1, 2, 3, 4, 5] (5 bytes)
        // Length prefix: 5 (u32)
        data.extend_from_slice(&5u32.to_ne_bytes());
        data.extend_from_slice(&[1, 2, 3, 4, 5]);
        data.extend_from_slice(&[0, 0, 0]); // Padding to 8 bytes
        data.extend_from_slice(b"next");

        let mut bytes_slice = data.as_slice();
        match deserialize_array(&mut bytes_slice).unwrap() {
            Argument::Array(arr) => assert_eq!(arr, vec![1, 2, 3, 4, 5]),
            _ => panic!("Wrong argument type"),
        }
        assert_eq!(bytes_slice, b"next");
    }

    #[test]
    fn test_parse_wl_display_sync() {
        let (pm, registry) = setup_test_protocols_and_registry();
        // wl_display.sync: sender_id=1, opcode=0, new_id callback_id=2, len=12
        let mut msg_bytes = Vec::new();
        msg_bytes.extend_from_slice(&1u32.to_ne_bytes()); // sender_id
        msg_bytes.extend_from_slice(&((0u32 << 16) | 12u32).to_ne_bytes()); // opcode 0, len 12
        msg_bytes.extend_from_slice(&2u32.to_ne_bytes()); // new_id (callback) = 2

        let (msg, rest) = parse_message(&msg_bytes, &pm, &registry, &mut None).unwrap();

        assert_eq!(msg.sender_id, 1);
        assert_eq!(msg.opcode, 0); // sync
        assert_eq!(msg.len, 12);
        assert_eq!(msg.args.len(), 1);
        assert_eq!(msg.args[0], Argument::NewId(2)); // callback id
        assert!(rest.is_empty());
    }

     #[test]
    fn test_parse_wl_display_get_registry() {
        let (pm, registry) = setup_test_protocols_and_registry();
        // wl_display.get_registry: sender_id=1, opcode=1, new_id registry_id=3, len=12
        let mut msg_bytes = Vec::new();
        msg_bytes.extend_from_slice(&1u32.to_ne_bytes()); // sender_id
        msg_bytes.extend_from_slice(&((1u32 << 16) | 12u32).to_ne_bytes()); // opcode 1, len 12
        msg_bytes.extend_from_slice(&3u32.to_ne_bytes()); // new_id (registry) = 3

        let (msg, rest) = parse_message(&msg_bytes, &pm, &registry, &mut None).unwrap();

        assert_eq!(msg.sender_id, 1);
        assert_eq!(msg.opcode, 1); // get_registry
        assert_eq!(msg.len, 12);
        assert_eq!(msg.args.len(), 1);
        assert_eq!(msg.args[0], Argument::NewId(3)); // registry id
        assert!(rest.is_empty());
    }

    #[test]
    fn test_parse_message_shm_create_pool_with_fd() {
        let (pm, mut registry) = setup_test_protocols_and_registry();
        // Manually register a wl_shm object for the test, assuming client created it with ID 2.
        let shm_object_id = 2u32;
        registry.new_object(
            1, // client_id
            shm_object_id,
            super::super::object_registry::WlDisplay {}, // Placeholder object, type doesn't matter for this test path
            "wl_shm".to_string(),
            1
        ).unwrap();

        // wl_shm.create_pool: sender_id=shm_object_id(2), opcode=0
        // args: new_id (pool_id)=3, fd=DUMMY_FD, size=4096
        // len = header(8) + new_id(4) + fd(0, ancillary) + size(4) = 16
        let mut msg_bytes = Vec::new();
        msg_bytes.extend_from_slice(&shm_object_id.to_ne_bytes()); // sender_id
        msg_bytes.extend_from_slice(&((0u32 << 16) | 16u32).to_ne_bytes()); // opcode 0, len 16
        msg_bytes.extend_from_slice(&3u32.to_ne_bytes()); // new_id (pool_id) = 3
        // FD is not in byte stream
        msg_bytes.extend_from_slice(&4096i32.to_ne_bytes()); // size = 4096

        let dummy_fd = 5; // Example FD
        let mut ancillary_fds = Some(vec![dummy_fd]); // deserialize_fd pops, so last FD for first arg.
                                                    // If create_pool expects FD as 2nd arg, and it's the only FD, this is fine.
                                                    // The spec for wl_shm.create_pool is (new_id, fd, size). FD is 2nd.
                                                    // If `pending_fds` in client.rs is reversed (as it is now),
                                                    // then `vec![dummy_fd]` results in `pop()` correctly getting `dummy_fd`.

        let (msg, rest) = parse_message(&msg_bytes, &pm, &registry, &mut ancillary_fds).unwrap();

        assert_eq!(msg.sender_id, shm_object_id);
        assert_eq!(msg.opcode, 0); // create_pool
        assert_eq!(msg.len, 16);
        assert_eq!(msg.args.len(), 3);
        assert_eq!(msg.args[0], Argument::NewId(3)); // pool_id
        assert_eq!(msg.args[1], Argument::Fd(dummy_fd)); // The FD
        assert_eq!(msg.args[2], Argument::Int(4096));  // size
        assert!(rest.is_empty());
        assert!(ancillary_fds.unwrap().is_empty(), "FD should have been consumed");
    }


    #[test]
    fn test_parse_message_not_enough_data_for_body() {
        let (pm, registry) = setup_test_protocols_and_registry();
        let mut msg_bytes = Vec::new();
        msg_bytes.extend_from_slice(&1u32.to_ne_bytes()); // sender_id = wl_display (1)
        msg_bytes.extend_from_slice(&((0u32 << 16) | 12u32).to_ne_bytes()); // opcode 0 (sync), len 12
        msg_bytes.extend_from_slice(&2u16.to_ne_bytes()); // Body: only 2 bytes, but NewId needs 4.

        let result = parse_message(&msg_bytes, &pm, &registry, &mut None);
        // The error should come from deserialize_new_id due to insufficient data for the argument.
        assert_matches!(result, Err(MessageParseError::NotEnoughData(_)));
    }

    #[test]
    fn test_parse_message_buffer_too_short_for_declared_len() {
        let (pm, registry) = setup_test_protocols_and_registry();
        let mut msg_bytes = Vec::new();
        msg_bytes.extend_from_slice(&1u32.to_ne_bytes());
        msg_bytes.extend_from_slice(&((0u32 << 16) | 12u32).to_ne_bytes()); // len 12, but buffer is only 8.

        let result = parse_message(&msg_bytes, &pm, &registry, &mut None);
        assert_matches!(result, Err(MessageParseError::NotEnoughData(msg)) if msg.contains("Message header indicates length 12, but buffer only has 8 bytes"));
    }

    #[test]
    fn test_fixed_point_conversion() {
        let f_val = 10.5; // 10.5 * 256 = 2688
        let fixed = Argument::f64_to_fixed(f_val);
        assert_eq!(fixed, 2688);
        assert_eq!(Argument::fixed_to_f64(fixed), f_val);

        let f_val_neg = -2.75; // -2.75 * 256 = -704
        let fixed_neg = Argument::f64_to_fixed(f_val_neg);
        assert_eq!(fixed_neg, -704);
        assert_eq!(Argument::fixed_to_f64(fixed_neg), f_val_neg);
    }

    #[test]
    fn test_deserialize_fd_success() {
        let fd1 = 5;
        let fd2 = 10;
        let mut available_fds = vec![fd2, fd1]; // Simulating FDs received, order might matter (e.g. pop vs remove)
                                               // If deserialize_fd uses pop, it gets fd1 then fd2.
                                               // If it uses remove(0), it gets fd2 then fd1.
                                               // Let's assume remove(0) for now, meaning FDs are in order of arguments.
                                               // To test pop(), reverse the order in `available_fds`.
                                               // Current implementation of deserialize_fd uses pop(). So fd1 then fd2.

        let mut dummy_buffer_slice = &[][..]; // Not used by deserialize_fd

        match deserialize_fd(&mut dummy_buffer_slice, &mut available_fds) {
            Ok(Argument::Fd(received_fd)) => assert_eq!(received_fd, fd1),
            _ => panic!("deserialize_fd failed or returned wrong type"),
        }
        assert_eq!(available_fds.len(), 1);
        assert_eq!(available_fds[0], fd2); // fd1 was consumed

        match deserialize_fd(&mut dummy_buffer_slice, &mut available_fds) {
            Ok(Argument::Fd(received_fd)) => assert_eq!(received_fd, fd2),
            _ => panic!("deserialize_fd failed or returned wrong type"),
        }
        assert!(available_fds.is_empty()); // All FDs consumed
    }

    #[test]
    fn test_deserialize_fd_not_enough_fds() {
        let mut available_fds: Vec<RawFd> = vec![];
        let mut dummy_buffer_slice = &[][..];

        match deserialize_fd(&mut dummy_buffer_slice, &mut available_fds) {
            Err(MessageParseError::InvalidArgument(msg)) => {
                assert!(msg.contains("Expected file descriptor, but none were available"));
            }
            _ => panic!("deserialize_fd should have failed with InvalidArgument"),
        }
    }

    // Example of how parse_message might be called with FDs (conceptual)
    // This test won't pass without a message type that actually expects an FD in its definition
    // within the hardcoded part of parse_message, or with dynamic protocol parsing.
    // The test `test_parse_message_shm_create_pool_with_fd` now covers FD argument parsing.
    /*
    #[test]
    fn test_parse_message_with_fd_argument() {
        // ... (conceptual test as before, now less needed due to specific test above) ...
    }
    */

    // --- Serialization Tests ---
    #[test]
    fn test_serialize_argument_simple_types() {
        let mut bytes = Vec::new();
        serialize_argument(&Argument::Int(-10), &mut bytes).unwrap();
        assert_eq!(bytes, (-10i32).to_ne_bytes().to_vec());
        bytes.clear();

        serialize_argument(&Argument::Uint(100), &mut bytes).unwrap();
        assert_eq!(bytes, 100u32.to_ne_bytes().to_vec());
        bytes.clear();

        serialize_argument(&Argument::Fixed(2560), &mut bytes).unwrap(); // 10.0 * 256
        assert_eq!(bytes, 2560i32.to_ne_bytes().to_vec());
        bytes.clear();

        serialize_argument(&Argument::Object(123), &mut bytes).unwrap();
        assert_eq!(bytes, 123u32.to_ne_bytes().to_vec());
        bytes.clear();

        serialize_argument(&Argument::NewId(456), &mut bytes).unwrap();
        assert_eq!(bytes, 456u32.to_ne_bytes().to_vec());
    }

    #[test]
    fn test_serialize_argument_string() {
        let mut bytes = Vec::new();
        serialize_argument(&Argument::String("hello".to_string()), &mut bytes).unwrap();

        let mut expected = Vec::new();
        expected.extend_from_slice(&(6u32).to_ne_bytes()); // len_with_null
        expected.extend_from_slice(b"hello\0");
        expected.extend_from_slice(&[0u8, 0u8]); // padding
        assert_eq!(bytes, expected);
    }

    #[test]
    fn test_serialize_argument_array() {
        let mut bytes = Vec::new();
        let arr_data = vec![1, 2, 3, 4, 5];
        serialize_argument(&Argument::Array(arr_data.clone()), &mut bytes).unwrap();

        let mut expected = Vec::new();
        expected.extend_from_slice(&(arr_data.len() as u32).to_ne_bytes());
        expected.extend_from_slice(&arr_data);
        expected.extend_from_slice(&[0u8, 0u8, 0u8]); // padding for 5 bytes data
        assert_eq!(bytes, expected);
    }

    #[test]
    fn test_serialize_argument_fd_error() {
        let mut bytes = Vec::new();
        let result = serialize_argument(&Argument::Fd(0), &mut bytes);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Argument::Fd cannot be serialized into the main byte stream. FDs must be passed via ancillary data.");
    }

    #[test]
    fn test_serialize_message_simple() {
        let sender_id = 1;
        let opcode = 0;
        let args = vec![Argument::NewId(2)]; // wl_display.sync example

        let bytes = serialize_message(sender_id, opcode, &args).unwrap();

        let mut expected = Vec::new();
        expected.extend_from_slice(&sender_id.to_ne_bytes());
        let total_len = 8 + 4; // header + u32
        let len_opcode = ((total_len as u32) << 16) | (opcode as u32);
        expected.extend_from_slice(&len_opcode.to_ne_bytes());
        expected.extend_from_slice(&2u32.to_ne_bytes()); // NewId(2)

        assert_eq!(bytes, expected);
    }

    #[test]
    fn test_serialize_message_with_string_and_padding() {
        let sender_id = 1;
        let opcode = 1; // wl_display.error (object_id, code, message)
                        // For this test, let's make object_id an Uint for simplicity of arg list
        let args = vec![
            Argument::Uint(123), // object_id that caused error
            Argument::Uint(5),   // error code
            Argument::String("short".to_string()), // 5 chars + null = 6 bytes, pads to 8
        ];

        let bytes = serialize_message(sender_id, opcode, &args).unwrap();

        let mut expected_arg_bytes = Vec::new();
        serialize_argument(&args[0], &mut expected_arg_bytes).unwrap();
        serialize_argument(&args[1], &mut expected_arg_bytes).unwrap();
        serialize_argument(&args[2], &mut expected_arg_bytes).unwrap();

        let total_len = 8 + expected_arg_bytes.len();
        let mut expected_header = Vec::new();
        expected_header.extend_from_slice(&sender_id.to_ne_bytes());
        let len_opcode = ((total_len as u32) << 16) | (opcode as u32);
        expected_header.extend_from_slice(&len_opcode.to_ne_bytes());

        let mut expected_total = expected_header;
        expected_total.append(&mut expected_arg_bytes);

        assert_eq!(bytes, expected_total);
        assert_eq!(total_len, 8 + 4 + 4 + (4 + 5 + 1 + 2)); // header + u32 + u32 + (len_str + str_data + null + pad)
    }
}
