use std::ffi::{CString, NulError};
use std::os::unix::io::RawFd; // For RawFd type, typically i32
use super::object::ObjectId; // Changed from crate::wayland::object

// --- Error Types ---
#[derive(Debug, PartialEq)]
pub enum SerializationError {
    StringContainsNul(NulError),
    // Could add BufferTooSmall if we pre-allocate and it's not enough
}

impl From<NulError> for SerializationError {
    fn from(err: NulError) -> Self {
        SerializationError::StringContainsNul(err)
    }
}

#[derive(Debug, PartialEq)]
pub enum DeserializationError {
    UnexpectedEof,
    InvalidString(std::str::Utf8Error),
    StringMissingNul,
    InvalidPadding,
    MessageTooShort, // Header indicates size larger than provided buffer
}

// --- Message Structures ---
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)] // Ensure layout is as expected for direct memory reads/writes
pub struct MessageHeader {
    pub object_id: ObjectId,
    pub size_opcode: u32, // size (upper 16 bits) and opcode (lower 16 bits)
}

impl MessageHeader {
    pub const SIZE: usize = std::mem::size_of::<MessageHeader>(); // Should be 8 bytes

    pub fn new(object_id: ObjectId, opcode: u16, size: u16) -> Self {
        MessageHeader {
            object_id,
            size_opcode: ((size as u32) << 16) | (opcode as u32),
        }
    }

    pub fn size(&self) -> u16 {
        (self.size_opcode >> 16) as u16
    }

    pub fn opcode(&self) -> u16 {
        (self.size_opcode & 0xFFFF) as u16
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WlArgument {
    Int(i32),
    Uint(u32),
    Fixed(i32), // 24.8 fixed point
    String(CString),
    Object(ObjectId),
    NewId(ObjectId), // Often a placeholder ID that the server replaces
    Array(Vec<u8>),
    Fd(RawFd),
}

impl WlArgument {
    pub fn size(&self) -> usize {
        match self {
            WlArgument::Int(_) => 4,
            WlArgument::Uint(_) => 4,
            WlArgument::Fixed(_) => 4,
            WlArgument::String(s) => {
                let len_with_nul = s.as_bytes_with_nul().len();
                (len_with_nul + 3) & !3 // Padded to 4-byte boundary
            }
            WlArgument::Object(_) => 4,
            WlArgument::NewId(_) => 4,
            WlArgument::Array(arr) => {
                4 + arr.len() + ((4 - (arr.len() % 4)) % 4) // size u32 + data + padding for data
            }
            WlArgument::Fd(_) => 4, // FDs are sent as u32/i32 on the wire
        }
    }
}

// --- Helper functions for reading/writing ---

fn write_u32(writer: &mut Vec<u8>, value: u32) {
    writer.extend_from_slice(&value.to_ne_bytes()); // Native endianness
}

fn write_i32(writer: &mut Vec<u8>, value: i32) {
    writer.extend_from_slice(&value.to_ne_bytes());
}

fn read_u32(buffer: &mut &[u8]) -> Result<u32, DeserializationError> {
    if buffer.len() < 4 {
        return Err(DeserializationError::UnexpectedEof);
    }
    let (val_bytes, rest) = buffer.split_at(4);
    *buffer = rest;
    Ok(u32::from_ne_bytes(val_bytes.try_into().unwrap())) // unwrap is safe due to length check
}

fn read_i32(buffer: &mut &[u8]) -> Result<i32, DeserializationError> {
    if buffer.len() < 4 {
        return Err(DeserializationError::UnexpectedEof);
    }
    let (val_bytes, rest) = buffer.split_at(4);
    *buffer = rest;
    Ok(i32::from_ne_bytes(val_bytes.try_into().unwrap()))
}


// --- Serialization ---
pub fn serialize_message(
    object_id: ObjectId,
    opcode: u16,
    args: &[WlArgument],
) -> Result<Vec<u8>, SerializationError> {
    let mut payload_size = 0;
    for arg in args {
        payload_size += arg.size();
    }

    let total_size = MessageHeader::SIZE + payload_size;
    if total_size > u16::MAX as usize {
        // This check is technically needed, though practically rare for single messages
        // Wayland message size is u16.
        // However, my MessageHeader.size is u16, and total_size is usize.
        // The actual size field in the header should be u16.
        panic!("Message too large for u16 size field"); // Or return an error
    }

    let header = MessageHeader::new(object_id, opcode, total_size as u16);
    let mut buffer = Vec::with_capacity(total_size);

    // Write header
    buffer.extend_from_slice(&header.object_id.to_ne_bytes());
    buffer.extend_from_slice(&header.size_opcode.to_ne_bytes());

    // Write arguments
    for arg in args {
        match arg {
            WlArgument::Int(val) => write_i32(&mut buffer, *val),
            WlArgument::Uint(val) => write_u32(&mut buffer, *val),
            WlArgument::Fixed(val) => write_i32(&mut buffer, *val),
            WlArgument::Object(id) => write_u32(&mut buffer, *id),
            WlArgument::NewId(id) => write_u32(&mut buffer, *id),
            WlArgument::Fd(fd) => write_i32(&mut buffer, *fd), // FDs are i32 on wire
            WlArgument::String(s_val) => {
                let bytes_with_nul = s_val.as_bytes_with_nul();
                let len_with_nul = bytes_with_nul.len();
                write_u32(&mut buffer, len_with_nul as u32); // String length including NUL
                buffer.extend_from_slice(bytes_with_nul);
                let padding = (4 - (len_with_nul % 4)) % 4;
                for _ in 0..padding {
                    buffer.push(0);
                }
            }
            WlArgument::Array(arr_val) => {
                write_u32(&mut buffer, arr_val.len() as u32);
                buffer.extend_from_slice(arr_val);
                let padding = (4 - (arr_val.len() % 4)) % 4;
                for _ in 0..padding {
                    buffer.push(0);
                }
            }
        }
    }
    Ok(buffer)
}

// --- Deserialization ---

// Argument types for guiding deserialization
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArgType {
    Int, Uint, Fixed, String, Object, NewId, Array, Fd,
}

pub fn deserialize_message(
    buffer: &mut &[u8], // Input buffer, will be consumed
    arg_signatures: &[ArgType], // Expected argument types
) -> Result<(MessageHeader, Vec<WlArgument>), DeserializationError> {
    if buffer.len() < MessageHeader::SIZE {
        return Err(DeserializationError::UnexpectedEof);
    }

    let header_object_id = read_u32(buffer)?;
    let header_size_opcode = read_u32(buffer)?;
    let header = MessageHeader { object_id: header_object_id, size_opcode: header_size_opcode };

    if header.size() < MessageHeader::SIZE as u16 {
        return Err(DeserializationError::MessageTooShort); // Header size implies less than header itself
    }

    // The remaining payload according to the header
    let mut payload_len = header.size() as usize - MessageHeader::SIZE;
    if buffer.len() < payload_len {
        return Err(DeserializationError::UnexpectedEof); // Buffer doesn't contain full payload
    }

    // For this function, we'll consume only up to payload_len from the original buffer slice
    // to avoid over-reading if the buffer is longer than one message.
    // However, the `buffer` slice itself is modified by read_ helpers.
    // A better way is to take a slice of the exact message size first.

    // Let's re-slice `buffer` to represent only the payload for this message.
    // This requires careful handling if `buffer` is supposed to be advanced past the whole message.
    // The current `read_u32` etc. advance the passed slice.
    // For now, we assume `buffer` is just one message or we are careful with its lifetime.

    let mut args = Vec::with_capacity(arg_signatures.len());

    // If arg_signatures is empty, and payload_len > 0, it's an error or unknown data
    // For now, strictly follow arg_signatures. If payload_len is not consumed fully, it's an issue.

    for arg_type in arg_signatures {
        if payload_len == 0 && arg_signatures.len() > args.len() {
             // Expected more args but no payload left
            return Err(DeserializationError::UnexpectedEof);
        }

        let initial_payload_len = payload_len;

        match arg_type {
            ArgType::Int => {
                if payload_len < 4 { return Err(DeserializationError::UnexpectedEof); }
                args.push(WlArgument::Int(read_i32(buffer)?));
                payload_len -= 4;
            }
            ArgType::Uint => {
                if payload_len < 4 { return Err(DeserializationError::UnexpectedEof); }
                args.push(WlArgument::Uint(read_u32(buffer)?));
                payload_len -= 4;
            }
            ArgType::Fixed => {
                if payload_len < 4 { return Err(DeserializationError::UnexpectedEof); }
                args.push(WlArgument::Fixed(read_i32(buffer)?));
                payload_len -= 4;
            }
            ArgType::Object => {
                if payload_len < 4 { return Err(DeserializationError::UnexpectedEof); }
                args.push(WlArgument::Object(read_u32(buffer)?));
                payload_len -= 4;
            }
            ArgType::NewId => {
                if payload_len < 4 { return Err(DeserializationError::UnexpectedEof); }
                args.push(WlArgument::NewId(read_u32(buffer)?));
                payload_len -= 4;
            }
            ArgType::Fd => {
                if payload_len < 4 { return Err(DeserializationError::UnexpectedEof); }
                args.push(WlArgument::Fd(read_i32(buffer)?));
                payload_len -= 4;
            }
            ArgType::String => {
                if payload_len < 4 { return Err(DeserializationError::UnexpectedEof); }
                let str_len = read_u32(buffer)? as usize;
                payload_len -= 4;

                if str_len == 0 { return Err(DeserializationError::StringMissingNul); } // Must have at least NUL
                if payload_len < str_len { return Err(DeserializationError::UnexpectedEof); }

                let total_padded_len = (str_len + 3) & !3;
                if payload_len < total_padded_len { return Err(DeserializationError::UnexpectedEof); }


                let str_bytes_with_nul = &buffer[..(str_len)];
                if str_bytes_with_nul.last() != Some(&0) {
                    return Err(DeserializationError::StringMissingNul);
                }

                // CString::new will fail if there are interior nuls, which is also invalid.
                // We take str_bytes_with_nul (which includes the one NUL) and CString::new will check it.
                // Or, more directly, from_vec_with_nul.
                match CString::new(&str_bytes_with_nul[..str_len-1]) { // Pass bytes *without* the nul
                    Ok(s) => args.push(WlArgument::String(s)),
                    Err(_) => return Err(DeserializationError::InvalidString(std::str::from_utf8(&[]).unwrap_err())), // Placeholder for NulError kind
                }

                *buffer = &buffer[total_padded_len..];
                payload_len -= total_padded_len;
            }
            ArgType::Array => {
                if payload_len < 4 { return Err(DeserializationError::UnexpectedEof); }
                let arr_data_len = read_u32(buffer)? as usize;
                payload_len -= 4;

                let total_padded_len = (arr_data_len + 3) & !3;
                if payload_len < total_padded_len { return Err(DeserializationError::UnexpectedEof); }

                let arr_data = buffer[..arr_data_len].to_vec();
                args.push(WlArgument::Array(arr_data));

                *buffer = &buffer[total_padded_len..];
                payload_len -= total_padded_len;
            }
        }
        if initial_payload_len < payload_len { // Should not happen if logic is correct
            panic!("Payload length increased during deserialization of an argument");
        }
    }

    if payload_len != 0 {
        // Data left over in payload after parsing all expected args, or not enough data consumed
        // This could be an InvalidData error or a sign of incorrect signature matching
        return Err(DeserializationError::InvalidData); // Or a more specific error
    }

    Ok((header, args))
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    // Helper to create a CString, panics on failure for tests
    fn cstr(s: &str) -> CString {
        CString::new(s).unwrap()
    }

    #[test]
    fn test_message_header_fields() {
        let id: ObjectId = 10;
        let opcode: u16 = 5;
        let size: u16 = 32;
        let header = MessageHeader::new(id, opcode, size);
        assert_eq!(header.object_id, id);
        assert_eq!(header.opcode(), opcode);
        assert_eq!(header.size(), size);
        assert_eq!(MessageHeader::SIZE, 8);
    }

    #[test]
    fn test_serialize_simple_message() {
        let object_id: ObjectId = 1;
        let opcode: u16 = 1;
        let args = [WlArgument::Int(123), WlArgument::Uint(456)];

        let total_arg_size: usize = args.iter().map(|a| a.size()).sum();
        let expected_size = (MessageHeader::SIZE + total_arg_size) as u16;

        let buffer = serialize_message(object_id, opcode, &args).unwrap();

        let mut read_buf = buffer.as_slice();
        let de_header_obj_id = read_u32(&mut read_buf).unwrap();
        let de_header_size_opcode = read_u32(&mut read_buf).unwrap();
        let de_header = MessageHeader{object_id: de_header_obj_id, size_opcode: de_header_size_opcode};

        assert_eq!(de_header.object_id, object_id);
        assert_eq!(de_header.opcode(), opcode);
        assert_eq!(de_header.size(), expected_size);
        assert_eq!(buffer.len(), expected_size as usize);

        let val1 = read_i32(&mut read_buf).unwrap();
        assert_eq!(val1, 123);
        let val2 = read_u32(&mut read_buf).unwrap();
        assert_eq!(val2, 456);
    }

    #[test]
    fn test_serialize_deserialize_round_trip() {
        let object_id: ObjectId = 42;
        let opcode: u16 = 7;
        let args = vec![
            WlArgument::Int(-100),
            WlArgument::Uint(200),
            WlArgument::Fixed(256 * 50), // 50.0 in 24.8 fixed point
            WlArgument::Object(1001),
            WlArgument::NewId(1002),
            WlArgument::Fd(3), // Mock FD
        ];
        let arg_types: Vec<ArgType> = args.iter().map(|arg| match arg {
            WlArgument::Int(_) => ArgType::Int,
            WlArgument::Uint(_) => ArgType::Uint,
            WlArgument::Fixed(_) => ArgType::Fixed,
            WlArgument::Object(_) => ArgType::Object,
            WlArgument::NewId(_) => ArgType::NewId,
            WlArgument::Fd(_) => ArgType::Fd,
            _ => panic!("Unsupported arg type in this test"),
        }).collect();

        let buffer = serialize_message(object_id, opcode, &args).unwrap();
        let (header, deserialized_args) = deserialize_message(&mut buffer.as_slice(), &arg_types).unwrap();

        assert_eq!(header.object_id, object_id);
        assert_eq!(header.opcode(), opcode);
        assert_eq!(header.size() as usize, MessageHeader::SIZE + args.iter().map(|a|a.size()).sum::<usize>());
        assert_eq!(args, deserialized_args);
    }

    #[test]
    fn test_string_serialization_padding() {
        let s = cstr("hello"); // 5 chars + NUL = 6 bytes. Padded to 8 bytes.
        let arg = WlArgument::String(s.clone());
        assert_eq!(arg.size(), 4 + 8); // size_u32 + padded_string_len (8 bytes for "hello\0")

        let object_id: ObjectId = 1;
        let opcode: u16 = 1;
        let args = [arg];

        let buffer = serialize_message(object_id, opcode, &args).unwrap();
        // Expected: obj_id (4), size_op (4), str_len_u32 (4), "hello\0" (6), padding (2) = 20
        assert_eq!(buffer.len(), MessageHeader::SIZE + 4 + ((s.as_bytes_with_nul().len() + 3) & !3) );
        assert_eq!(buffer.len(), 8 + 4 + 8); // Header + len_u32 + ("hello\0" + 2 pad) = 20

        let mut read_buf = buffer.as_slice();
        let (_header, deserialized_args) = deserialize_message(&mut read_buf, &[ArgType::String]).unwrap();

        assert_eq!(deserialized_args.len(), 1);
        match &deserialized_args[0] {
            WlArgument::String(ds) => assert_eq!(ds, &s),
            _ => panic!("Incorrect deserialized type"),
        }
    }

    #[test]
    fn test_string_serialization_exact_multiple_of_4() {
        let s = cstr("abc"); // 3 chars + NUL = 4 bytes. Padded to 4 bytes (no extra padding).
        let arg = WlArgument::String(s.clone());
        assert_eq!(arg.size(), 4 + 4); // size_u32 + string_len_with_nul (4 bytes for "abc\0")

        let buffer = serialize_message(1, 1, &[arg]).unwrap();
        // Expected: header (8) + str_len_u32 (4) + "abc\0" (4) = 16
        assert_eq!(buffer.len(), 16);

        let mut read_buf = buffer.as_slice();
        let (_header, deserialized_args) = deserialize_message(&mut read_buf, &[ArgType::String]).unwrap();
        assert_eq!(deserialized_args[0], WlArgument::String(s));
    }


    #[test]
    fn test_array_serialization_padding() {
        let data = vec![1, 2, 3, 4, 5]; // 5 bytes. Padded to 8 bytes.
        let arg = WlArgument::Array(data.clone());
        // Expected size: u32_len (4) + data_len (5) + padding (3) = 12
        assert_eq!(arg.size(), 4 + 5 + 3);

        let buffer = serialize_message(1, 1, &[arg]).unwrap();
        // Expected total: header(8) + arg_size(12) = 20
        assert_eq!(buffer.len(), 20);

        let mut read_buf = buffer.as_slice();
        let (_header, deserialized_args) = deserialize_message(&mut read_buf, &[ArgType::Array]).unwrap();
        assert_eq!(deserialized_args[0], WlArgument::Array(data));
    }

    #[test]
    fn test_array_serialization_exact_multiple_of_4() {
        let data = vec![1, 2, 3, 4]; // 4 bytes. Padded to 4 bytes (no extra padding).
        let arg = WlArgument::Array(data.clone());
        // Expected size: u32_len (4) + data_len (4) + padding (0) = 8
        assert_eq!(arg.size(), 4 + 4 + 0);

        let buffer = serialize_message(1, 1, &[arg]).unwrap();
        // Expected total: header(8) + arg_size(8) = 16
        assert_eq!(buffer.len(), 16);

        let mut read_buf = buffer.as_slice();
        let (_header, deserialized_args) = deserialize_message(&mut read_buf, &[ArgType::Array]).unwrap();
        assert_eq!(deserialized_args[0], WlArgument::Array(data));
    }


    #[test]
    fn test_deserialize_empty_args() {
        let object_id: ObjectId = 1;
        let opcode: u16 = 0;
        let args: Vec<WlArgument> = vec![];
        let arg_types: Vec<ArgType> = vec![];

        let buffer = serialize_message(object_id, opcode, &args).unwrap();
        assert_eq!(buffer.len(), MessageHeader::SIZE);

        let (header, deserialized_args) = deserialize_message(&mut buffer.as_slice(), &arg_types).unwrap();
        assert_eq!(header.object_id, object_id);
        assert_eq!(header.opcode(), opcode);
        assert_eq!(header.size() as usize, MessageHeader::SIZE);
        assert!(deserialized_args.is_empty());
    }

    #[test]
    fn test_deserialize_unexpected_eof_header() {
        let mut buffer = vec![1, 0, 0, 0]; // Too short for a header
        let result = deserialize_message(&mut buffer.as_slice(), &[]);
        assert_eq!(result, Err(DeserializationError::UnexpectedEof));
    }

    #[test]
    fn test_deserialize_message_too_short_according_to_header() {
        // Header indicates size 32, but only header is present (8 bytes)
        let header = MessageHeader::new(1, 1, 32);
        let mut buffer = Vec::new();
        write_u32(&mut buffer, header.object_id);
        write_u32(&mut buffer, header.size_opcode);
        // buffer is now 8 bytes long

        let result = deserialize_message(&mut buffer.as_slice(), &[ArgType::Int]);
        assert_eq!(result, Err(DeserializationError::UnexpectedEof)); // Not MessageTooShort, because buffer itself is too short
    }

    #[test]
    fn test_deserialize_string_missing_nul_in_data() {
        let header = MessageHeader::new(1,1, 8 + 4 + 4); // Header + len_u32 + "test" (no NUL)
        let mut buffer = Vec::new();
        write_u32(&mut buffer, header.object_id);
        write_u32(&mut buffer, header.size_opcode);
        write_u32(&mut buffer, 4); // String length
        buffer.extend_from_slice(b"test"); // No NUL terminator

        let result = deserialize_message(&mut buffer.as_slice(), &[ArgType::String]);
        assert_eq!(result, Err(DeserializationError::StringMissingNul));
    }

    #[test]
    fn test_deserialize_payload_remaining_data() {
        // Message with one int, but buffer has extra data that's part of this message's payload
        // according to header size, but not consumed by arg_signatures.
        let mut serialized_payload = Vec::new();
        write_i32(&mut serialized_payload, 123); // The int argument
        write_i32(&mut serialized_payload, 456); // Extra data

        let header = MessageHeader::new(1, 1, (MessageHeader::SIZE + serialized_payload.len()) as u16);

        let mut buffer = Vec::new();
        write_u32(&mut buffer, header.object_id);
        write_u32(&mut buffer, header.size_opcode);
        buffer.extend_from_slice(&serialized_payload);

        // We only expect one Int. deserialize_message should find leftover data.
        let result = deserialize_message(&mut buffer.as_slice(), &[ArgType::Int]);
        assert_eq!(result, Err(DeserializationError::InvalidData));
    }

    #[test]
    fn test_deserialize_insufficient_data_for_arg() {
        // Header for an int, but payload is too short
        let header = MessageHeader::new(1, 1, (MessageHeader::SIZE + 4) as u16); // Expects 4 bytes payload
        let mut buffer = Vec::new();
        write_u32(&mut buffer, header.object_id);
        write_u32(&mut buffer, header.size_opcode);
        buffer.push(1); // Only 1 byte of payload

        let result = deserialize_message(&mut buffer.as_slice(), &[ArgType::Int]);
        assert_eq!(result, Err(DeserializationError::UnexpectedEof));
    }

     #[test]
    fn test_deserialize_string_payload_too_short_for_padded_len() {
        let mut buffer = Vec::new();
        // Header: obj_id=1, opcode=1, size = (8 header + 4 strlen + 5 "hell\0" + 3 pad = 20 bytes)
        let header = MessageHeader::new(1, 1, 20);
        write_u32(&mut buffer, header.object_id);
        write_u32(&mut buffer, header.size_opcode);
        write_u32(&mut buffer, 5); // String has 5 bytes ("hell\0")
        buffer.extend_from_slice(b"hell\0"); // 5 bytes
        // Missing 3 bytes of padding from the buffer that header.size accounts for
        // buffer.extend_from_slice(&[0,0,0]);

        // The buffer slice passed to deserialize_message will be shorter than header.size() implies
        // for the payload part.
        // deserialize_message reads header (8b), then string length (4b -> 5).
        // Payload should be 5 (string) + 3 (padding) = 8 bytes.
        // Buffer has 5 bytes of payload. So, UnexpectedEof when trying to read padding.
        let result = deserialize_message(&mut buffer.as_slice(), &[ArgType::String]);
        assert_eq!(result, Err(DeserializationError::UnexpectedEof));
    }
}
