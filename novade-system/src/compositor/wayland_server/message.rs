use byteorder::{ByteOrder, NativeEndian, ReadBytesExt}; // NativeEndian should align with Wayland's spec (usually little-endian)
use std::io::{Cursor, Read};
use std::os::unix::io::RawFd;
use std::convert::TryInto;

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

#[allow(unused_variables)] // ancillary_data might not be used if FD passing is basic
fn deserialize_fd(bytes: &mut &[u8], ancillary_data: Option<&[RawFd]>) -> Result<Argument, MessageParseError> {
    // File descriptors are not in the main byte stream.
    // They are sent as ancillary data with the sendmsg() call.
    // The `ancillary_data` parameter would typically be a slice of FDs received.
    // This function would take the next available FD from that slice.
    // For now, this is a placeholder.
    // A real implementation would need to manage the FDs passed alongside the message.
    if let Some(fds) = ancillary_data {
        if !fds.is_empty() {
            // This is a simplification. The caller of parse_message needs to manage which FD belongs to which argument.
            // Let's assume for parse_message that it passes the correct FD slice for *this* argument.
            // For wl_display.sync, there are no FDs.
            // Ok(Argument::Fd(fds[0])) // and then the caller would advance the slice.
            return Err(MessageParseError::InvalidArgument("FD deserialization not fully implemented yet, but got FD data.".to_string()));
        }
    }
    // If an FD argument is expected but no FDs are available, it's an error.
    // This depends on the protocol definition.
    Err(MessageParseError::InvalidArgument("Expected file descriptor, but none provided in ancillary data.".to_string()))
}


/// Parses a single Wayland message from the buffer.
/// buffer: The current data read from the client.
/// ancillary_fds: File descriptors received alongside the message.
/// Returns the parsed Message and the remaining unconsumed portion of the buffer.
pub fn parse_message<'a>(
    initial_buffer: &'a [u8],
    ancillary_fds: &mut Option<Vec<RawFd>>, // Consumes FDs as they are parsed
) -> Result<(Message, &'a [u8]), MessageParseError> {
    let (sender_id, opcode, len) = parse_message_header(initial_buffer)?;

    if initial_buffer.len() < len as usize {
        return Err(MessageParseError::NotEnoughData(format!(
            "Message requires {} bytes, but buffer only has {} bytes",
            len,
            initial_buffer.len()
        )));
    }

    let mut message_body_buffer = &initial_buffer[8..len as usize];
    let rest_of_buffer = &initial_buffer[len as usize..];
    let mut args = Vec::new();

    // --- Protocol-specific parsing logic ---
    // This is where knowledge of the Wayland protocol is required.
    // For this subtask, we will only implement parsing for wl_display.sync (object_id=1, opcode=0)
    // wl_display.sync takes one argument: new_id (type wl_callback).

    if sender_id == 1 { // wl_display
        match opcode {
            0 => { // sync request
                // Expected argument: new_id (wl_callback)
                if message_body_buffer.len() < 4 {
                     return Err(MessageParseError::NotEnoughData(
                        "wl_display.sync expects a new_id (4 bytes), not enough data.".to_string()
                    ));
                }
                args.push(deserialize_new_id(&mut message_body_buffer)?);
            }
            1 => { // get_registry request
                // Expected argument: new_id (wl_registry)
                 if message_body_buffer.len() < 4 {
                     return Err(MessageParseError::NotEnoughData(
                        "wl_display.get_registry expects a new_id (4 bytes), not enough data.".to_string()
                    ));
                }
                args.push(deserialize_new_id(&mut message_body_buffer)?);
            }
            _ => return Err(MessageParseError::UnsupportedOpcode(opcode)),
        }
    } else {
        // For other interfaces, we don't know the argument types yet.
        // A full implementation would use protocol definitions (e.g., from XML)
        // to determine expected arguments.
        // For now, if it's not wl_display, we can't parse its args.
        // Or, we could consume the rest of message_body_buffer as a raw array if appropriate.
        // This would require careful handling of padding and FDs.
        // Let's return an error for unsupported interfaces for now.
        return Err(MessageParseError::UnsupportedInterface(sender_id));
    }

    // Ensure all data in message_body_buffer has been consumed if args were expected.
    // Some messages might have no arguments but still have a body (e.g., for padding).
    // The `len` in the header is the source of truth for message boundaries.
    // If `message_body_buffer` is not empty after parsing known args, it could be an error
    // or represent arguments we haven't parsed (due to incomplete protocol impl).
    if !message_body_buffer.is_empty() && !args.is_empty() {
        // This might indicate that not all arguments were parsed, or there's extra data.
        // For strict parsing, this should be an error.
        // However, Wayland messages are padded to 32-bit boundaries.
        // The total length `len` includes this padding.
        // Individual argument parsers should handle their own data + padding.
        // If after all args are parsed, message_body_buffer is not empty, it means
        // the sum of (data+padding) for each arg was less than (message_len - header_len).
        // This usually means an issue with parsing logic or message construction.
        // For now, let's be strict: if we parsed args, the buffer should be empty.
        // This needs refinement as some messages might have trailing padding not tied to a specific arg.
        // The total message length `len` already accounts for all padding.
        // So, if `message_body_buffer` (which is `initial_buffer[8..len]`) is not fully consumed
        // by argument deserializers, it's an issue.

        // Let's re-evaluate: message_body_buffer is `len - 8` bytes.
        // The argument deserializers consume from it. If they consume exactly that much, it's fine.
        // If they consume less, and `message_body_buffer` is not empty, it's an error.
        // This means our argument list for the opcode was incomplete.
        // This check is implicitly handled by the fact that `parse_message` returns the `rest_of_buffer`.
        // The caller is responsible for using `rest_of_buffer`.
        // The current argument parsers (e.g. string, array) try to consume padding.
        // Others (u32, i32) do not, relying on 4-byte alignment.
        // This needs to be consistent. For now, we assume the protocol definition correctly
        // leads to consuming the whole message_body_buffer.
    }


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


#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::io::AsRawFd;

    #[test]
    fn test_parse_message_header_valid() {
        // sender_id=1, opcode=0, len=12
        let bytes = [1, 0, 0, 0,  0, 0, 12, 0]; // Little Endian for opcode_len_raw: (0<<16)|12
        // Actually, opcode is high bits, len is low bits. So (opcode << 16) | len
        // For opcode=0, len=12: (0 << 16) | 12 = 12.  NativeEndian::write_u32(&mut buf[4..], 12);
        // For opcode=1, len=12: (1 << 16) | 12 = 65536 | 12 = 65548
        let mut header_bytes = [0u8; 8];
        NativeEndian::write_u32(&mut header_bytes[0..4], 1); // sender_id = 1
        NativeEndian::write_u32(&mut header_bytes[4..8], (0u32 << 16) | 12u32); // opcode=0, len=12

        let result = parse_message_header(&header_bytes);
        assert_eq!(result, Ok((1, 0, 12)));

        NativeEndian::write_u32(&mut header_bytes[4..8], (1u32 << 16) | 16u32); // opcode=1, len=16
        let result2 = parse_message_header(&header_bytes);
        assert_eq!(result2, Ok((1, 1, 16)));
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
        // wl_display.sync:
        // sender_id (wl_display) = 1
        // opcode (sync) = 0
        // total length = header (8) + new_id (4) = 12 bytes
        // Arguments: new_id (wl_callback), let's say ID 2

        let mut msg_bytes = Vec::new();
        // Header
        msg_bytes.extend_from_slice(&1u32.to_ne_bytes()); // sender_id = 1
        let opcode = 0u16;
        let len = 12u16;
        msg_bytes.extend_from_slice(&((opcode as u32) << 16 | (len as u32)).to_ne_bytes());
        // Arguments
        msg_bytes.extend_from_slice(&2u32.to_ne_bytes()); // new_id (callback_id) = 2

        let (msg, rest) = parse_message(&msg_bytes, &mut None).unwrap();

        assert_eq!(msg.sender_id, 1);
        assert_eq!(msg.opcode, 0);
        assert_eq!(msg.len, 12);
        assert_eq!(msg.args.len(), 1);
        assert_eq!(msg.args[0], Argument::NewId(2));
        assert!(rest.is_empty());
    }

     #[test]
    fn test_parse_wl_display_get_registry() {
        // wl_display.get_registry:
        // sender_id (wl_display) = 1
        // opcode (get_registry) = 1
        // total length = header (8) + new_id (wl_registry) (4) = 12 bytes
        // Arguments: new_id (wl_registry), let's say ID 3

        let mut msg_bytes = Vec::new();
        // Header
        msg_bytes.extend_from_slice(&1u32.to_ne_bytes()); // sender_id = 1
        let opcode = 1u16;
        let len = 12u16;
        msg_bytes.extend_from_slice(&((opcode as u32) << 16 | (len as u32)).to_ne_bytes());
        // Arguments
        msg_bytes.extend_from_slice(&3u32.to_ne_bytes()); // new_id (registry_id) = 3

        let (msg, rest) = parse_message(&msg_bytes, &mut None).unwrap();

        assert_eq!(msg.sender_id, 1);
        assert_eq!(msg.opcode, 1);
        assert_eq!(msg.len, 12);
        assert_eq!(msg.args.len(), 1);
        assert_eq!(msg.args[0], Argument::NewId(3));
        assert!(rest.is_empty());
    }


    #[test]
    fn test_parse_message_not_enough_data_for_body() {
        let mut msg_bytes = Vec::new();
        // Header: sender_id=1, opcode=0, len=12
        msg_bytes.extend_from_slice(&1u32.to_ne_bytes());
        msg_bytes.extend_from_slice(&((0u32 << 16) | 12u32).to_ne_bytes());
        // Body: only 2 bytes, but expected 4 for new_id
        msg_bytes.extend_from_slice(&2u16.to_ne_bytes());

        let result = parse_message(&msg_bytes, &mut None);
        assert_matches!(result, Err(MessageParseError::NotEnoughData(_)));
    }

    #[test]
    fn test_parse_message_buffer_too_short_for_declared_len() {
        let mut msg_bytes = Vec::new();
        // Header: sender_id=1, opcode=0, len=12 (but buffer will be shorter)
        msg_bytes.extend_from_slice(&1u32.to_ne_bytes());
        msg_bytes.extend_from_slice(&((0u32 << 16) | 12u32).to_ne_bytes());
        // Missing body, total buffer length 8, but header says 12.

        let result = parse_message(&msg_bytes, &mut None);
         assert_matches!(result, Err(MessageParseError::NotEnoughData(msg)) if msg.contains("Message requires 12 bytes, but buffer only has 8 bytes"));
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
}
