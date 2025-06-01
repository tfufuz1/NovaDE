// In novade-system/src/compositor/wayland_server/protocol.rs

use crate::compositor::wayland_server::error::WaylandServerError;
use bytes::{Bytes, Buf, BytesMut};
use std::convert::TryInto;
use std::os::unix::io::RawFd;
use std::collections::HashMap; // For mock signature store
// Removed unused imports for ClientObjectSpace and ObjectEntry for now, will be needed for deeper validation
// use crate::compositor::wayland_server::objects::{Interface, ObjectEntry, ClientObjectSpace};
use tracing::{error, debug, trace, warn};


// === START: Existing content from protocol.rs (actual full content here) ===
pub const MESSAGE_HEADER_SIZE: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(u32);
impl ObjectId {
    pub fn new(id: u32) -> Self { ObjectId(id) }
    pub fn value(&self) -> u32 { self.0 }
    pub fn is_null(&self) -> bool { self.0 == 0 }
}
impl From<u32> for ObjectId { fn from(id: u32) -> Self { ObjectId(id) } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NewId(u32);
impl NewId {
    pub fn new(id: u32) -> Self { NewId(id) }
    pub fn value(&self) -> u32 { self.0 }
}
impl From<u32> for NewId { fn from(id: u32) -> Self { NewId(id) } }

#[derive(Debug, Clone)]
pub struct MessageHeader {
    pub object_id: ObjectId,
    pub size: u16,
    pub opcode: u16,
}

#[derive(Debug, Clone)]
pub struct RawMessage {
    pub header: MessageHeader,
    pub content: Bytes,
    pub fds: Vec<RawFd>,
}

#[derive(Debug, Clone)]
pub enum ArgumentValue {
    Int(i32), Uint(u32), Fixed(f64), String(String),
    Object(ObjectId), NewId(NewId), Array(Bytes), Fd(RawFd),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaylandWireType {
    Int, Uint, Fixed, String, Object, NewId, Array, Fd,
}

impl MessageHeader {
    pub fn parse(buffer: &mut Bytes) -> Result<Self, WaylandServerError> {
        if buffer.len() < MESSAGE_HEADER_SIZE {
            trace!("Buffer too small for message header: {} bytes", buffer.len());
            return Err(WaylandServerError::Protocol("Buffer too small for message header".to_string()));
        }
        let object_id_raw = buffer.get_u32_le();
        let size_opcode_raw = buffer.get_u32_le();
        let object_id = ObjectId::new(object_id_raw);
        let size = (size_opcode_raw >> 16) as u16;
        let opcode = (size_opcode_raw & 0xFFFF) as u16;
        if size < (MESSAGE_HEADER_SIZE as u16) {
            error!("Invalid message size in header: {} (must be at least {})", size, MESSAGE_HEADER_SIZE);
            return Err(WaylandServerError::Protocol(format!("Invalid message size in header: {}", size)));
        }
        debug!("Parsed message header: ObjectId({}), Size: {}, Opcode: {}", object_id.value(), size, opcode);
        Ok(MessageHeader { object_id, size, opcode })
    }
}

pub struct MessageParser {
    buffer: Bytes,
    current_message_fds: Vec<RawFd>,
}

// === END: Existing content from protocol.rs (actual full content here) ===


// New structures for message validation
#[derive(Debug, Clone)]
pub struct MessageSignature {
    pub interface_name: String,
    pub message_name: String,
    pub opcode: u16,
    pub since_version: u32,
    pub arg_types: Vec<WaylandWireType>,
}

pub type ProtocolSpecStore = HashMap<(String, u16), MessageSignature>;

pub fn mock_spec_store() -> ProtocolSpecStore {
    let mut store = HashMap::new();
    store.insert(("wl_compositor".to_string(), 0), MessageSignature {
        interface_name: "wl_compositor".to_string(),
        message_name: "create_surface".to_string(),
        opcode: 0,
        since_version: 1,
        arg_types: vec![WaylandWireType::NewId],
    });
    store.insert(("wl_surface".to_string(), 0), MessageSignature {
        interface_name: "wl_surface".to_string(),
        message_name: "attach".to_string(),
        opcode: 0,
        since_version: 1,
        arg_types: vec![WaylandWireType::Object, WaylandWireType::Int, WaylandWireType::Int],
    });
    store.insert(("wl_surface".to_string(), 1), MessageSignature {
        interface_name: "wl_surface".to_string(),
        message_name: "damage".to_string(),
        opcode: 1,
        since_version: 1,
        arg_types: vec![WaylandWireType::Int, WaylandWireType::Int, WaylandWireType::Int, WaylandWireType::Int],
    });
    store.insert(("wl_shm_pool".to_string(), 0), MessageSignature {
        interface_name: "wl_shm_pool".to_string(),
        message_name: "create_buffer".to_string(),
        opcode: 0,
        since_version: 1,
        arg_types: vec![
            WaylandWireType::NewId, WaylandWireType::Fd, WaylandWireType::Int, WaylandWireType::Int,
            WaylandWireType::Int, WaylandWireType::Int, WaylandWireType::Uint,
        ],
    });

    // wl_display.sync (opcode 0): new_id (callback)
    store.insert(("wl_display".to_string(), 0), MessageSignature {
        interface_name: "wl_display".to_string(), message_name: "sync".to_string(),
        opcode: 0, since_version: 1, arg_types: vec![WaylandWireType::NewId],
    });
    // wl_display.get_registry (opcode 1): new_id (registry)
    store.insert(("wl_display".to_string(), 1), MessageSignature {
        interface_name: "wl_display".to_string(), message_name: "get_registry".to_string(),
        opcode: 1, since_version: 1, arg_types: vec![WaylandWireType::NewId],
    });

    // wl_compositor.create_surface (opcode 0): new_id (surface)
    store.insert(("wl_compositor".to_string(), 0), MessageSignature {
        interface_name: "wl_compositor".to_string(),
        message_name: "create_surface".to_string(),
        opcode: 0, // REQ_CREATE_SURFACE_OPCODE from wl_compositor.rs
        since_version: 1, // Assuming version 1 for this core functionality
        arg_types: vec![WaylandWireType::NewId], // Expects one argument: new_id for the wl_surface
    });

    // wl_registry.bind (opcode 0): uint (name), new_id (id)
    store.insert(("wl_registry".to_string(), 0), MessageSignature {
        interface_name: "wl_registry".to_string(),
        message_name: "bind".to_string(),
        opcode: 0, // REQ_BIND_OPCODE from wl_registry.rs
        since_version: 1,
        arg_types: vec![WaylandWireType::Uint, WaylandWireType::NewId], // name, id<interface>
                                                                    // Note: The 'interface' (string) and 'version' (uint)
                                                                    // that are conceptually part of 'new_id<interface>'
                                                                    // are *not* separate arguments for wl_registry.bind.
                                                                    // Client libraries handle this. Our parser gives NewId(numeric_id).
                                                                    // The dispatcher uses context (the global being bound) for interface/version.
    });

    // wl_shm.create_pool (opcode 0): new_id (pool_id), fd (fd), int (size)
    store.insert(("wl_shm".to_string(), 0), MessageSignature {
        interface_name: "wl_shm".to_string(),
        message_name: "create_pool".to_string(),
        opcode: 0, // REQ_CREATE_POOL_OPCODE from wl_shm.rs
        since_version: 1,
        arg_types: vec![WaylandWireType::NewId, WaylandWireType::Fd, WaylandWireType::Int],
    });

    store
}


impl MessageParser {
    pub fn new() -> Self { // Re-define new here as it's part of the impl block
        MessageParser {
            buffer: Bytes::new(),
            current_message_fds: Vec::new(),
        }
    }

    pub fn append_data(&mut self, data: &[u8], fds: Vec<RawFd>) { // Re-define here
        let mut new_buf = Vec::with_capacity(self.buffer.len() + data.len());
        new_buf.extend_from_slice(&self.buffer);
        new_buf.extend_from_slice(data);
        self.buffer = Bytes::from(new_buf);
        self.current_message_fds.extend(fds);
        debug!("Appended {} bytes and {} FDs to parser. Total buffer size: {}", data.len(), self.current_message_fds.len(), self.buffer.len());
    }

    pub fn next_message(&mut self) -> Result<Option<RawMessage>, WaylandServerError> { // Re-define here
        if self.buffer.len() < MESSAGE_HEADER_SIZE {
            trace!("Parser buffer too small for a header ({} bytes). Waiting for more data.", self.buffer.len());
            return Ok(None);
        }
        let mut header_peek_buf = self.buffer.slice(0..MESSAGE_HEADER_SIZE);
        let _object_id_raw_peek = header_peek_buf.get_u32_le();
        let size_opcode_raw_peek = header_peek_buf.get_u32_le();
        let size_peek = (size_opcode_raw_peek >> 16) as u16;
        if size_peek < (MESSAGE_HEADER_SIZE as u16) {
            error!("Invalid message size in peeked header: {} (must be at least {})", size_peek, MESSAGE_HEADER_SIZE);
            self.buffer.clear();
            self.current_message_fds.clear();
            return Err(WaylandServerError::Protocol(format!("Invalid message size in peeked header: {}", size_peek)));
        }
        if self.buffer.len() < size_peek as usize {
            trace!("Partial message in buffer: Expected {} bytes, have {}. Waiting for more data.", size_peek, self.buffer.len());
            return Ok(None);
        }
        let header = MessageHeader::parse(&mut self.buffer)?;
        let content_size = header.size as usize - MESSAGE_HEADER_SIZE;
        let message_content = self.buffer.split_to(content_size);
        let fds_for_this_message = std::mem::take(&mut self.current_message_fds);
        debug!("Successfully parsed one message. Content size: {}. FDs: {}. Remaining buffer size: {}", message_content.len(), fds_for_this_message.len(), self.buffer.len());
        Ok(Some(RawMessage { header, content: message_content, fds: fds_for_this_message }))
    }

    pub fn validate_and_parse_args(
        raw_message: &RawMessage,
        signature: &MessageSignature,
    ) -> Result<Vec<ArgumentValue>, WaylandServerError> {

        let mut parsed_args = Vec::with_capacity(signature.arg_types.len());
        let mut current_content = raw_message.content.clone();
        let mut current_fds = raw_message.fds.clone();

        for (_i, arg_type) in signature.arg_types.iter().enumerate() {
            let arg_value = match arg_type {
                WaylandWireType::Int => ArgumentValue::Int(Self::read_i32(&mut current_content)?),
                WaylandWireType::Uint => ArgumentValue::Uint(Self::read_u32(&mut current_content)?),
                WaylandWireType::Fixed => ArgumentValue::Fixed(Self::read_fixed(&mut current_content)?),
                WaylandWireType::String => ArgumentValue::String(Self::read_string(&mut current_content)?),
                WaylandWireType::Object => {
                    ArgumentValue::Object(Self::read_object_id(&mut current_content)?)
                }
                WaylandWireType::NewId => ArgumentValue::NewId(Self::read_new_id(&mut current_content)?),
                WaylandWireType::Array => ArgumentValue::Array(Self::read_array(&mut current_content)?),
                WaylandWireType::Fd => ArgumentValue::Fd(Self::read_fd(&mut current_content, &mut current_fds)?),
            };
            parsed_args.push(arg_value);
        }

        if current_content.has_remaining() {
            warn!(
                "Message {}(opcode {}) for object {} had {} leftover bytes after parsing arguments.",
                signature.message_name, signature.opcode, raw_message.header.object_id.value(), current_content.remaining()
            );
            return Err(WaylandServerError::Protocol(format!(
                "Too much data for message {}::{}: {} unexpected leftover bytes.",
                signature.interface_name, signature.message_name, current_content.remaining()
            )));
        }

        if !current_fds.is_empty() {
            warn!(
                "Message {}(opcode {}) for object {} had {} leftover FDs after parsing arguments.",
                signature.message_name, signature.opcode, raw_message.header.object_id.value(), current_fds.len()
            );
            return Err(WaylandServerError::Protocol(format!(
                "Too many FDs for message {}::{}: {} unexpected leftover FDs.",
                signature.interface_name, signature.message_name, current_fds.len()
            )));
        }
        Ok(parsed_args)
    }

    pub fn read_u32(buf: &mut Bytes) -> Result<u32, WaylandServerError> {
        if buf.remaining() < 4 { return Err(WaylandServerError::Protocol("Buffer too small for u32".to_string()));}
        Ok(buf.get_u32_le())
    }
    pub fn read_i32(buf: &mut Bytes) -> Result<i32, WaylandServerError> {
        if buf.remaining() < 4 { return Err(WaylandServerError::Protocol("Buffer too small for i32".to_string()));}
        Ok(buf.get_i32_le())
    }
    pub fn read_fixed(buf: &mut Bytes) -> Result<f64, WaylandServerError> {
        Ok(Self::read_i32(buf)? as f64 / 256.0)
    }
    pub fn read_string(buf: &mut Bytes) -> Result<String, WaylandServerError> {
        let len = Self::read_u32(buf)? as usize;
        const MAX_STRING_LEN: usize = 4096;
        if len == 0 { return Err(WaylandServerError::Protocol("String length cannot be zero".to_string())); }
        if len > MAX_STRING_LEN { return Err(WaylandServerError::Protocol(format!("String length {} exceeds maximum {}", len, MAX_STRING_LEN))); }
        if buf.remaining() < len { return Err(WaylandServerError::Protocol(format!("Buffer too small for string of specified length {}", len))); }
        let str_bytes = buf.slice(0..len-1);
        let s = String::from_utf8(str_bytes.to_vec()).map_err(|e| WaylandServerError::Protocol(format!("Invalid UTF-8 string: {}", e)))?;
        buf.advance(len-1);
        if buf.remaining() < 1 || buf.get_u8() != 0 { return Err(WaylandServerError::Protocol("String not null-terminated correctly".to_string())); }
        let padding = (4 - (len % 4)) % 4;
        if buf.remaining() < padding { return Err(WaylandServerError::Protocol("Buffer too small for string padding".to_string())); }
        buf.advance(padding); Ok(s)
    }
    pub fn read_object_id(buf: &mut Bytes) -> Result<ObjectId, WaylandServerError> { Ok(ObjectId::new(Self::read_u32(buf)?)) }
    pub fn read_new_id(buf: &mut Bytes) -> Result<NewId, WaylandServerError> { Ok(NewId::new(Self::read_u32(buf)?)) }
    pub fn read_array(buf: &mut Bytes) -> Result<Bytes, WaylandServerError> {
        let len = Self::read_u32(buf)? as usize;
        const MAX_ARRAY_LEN: usize = 1024 * 1024;
        if len > MAX_ARRAY_LEN { return Err(WaylandServerError::Protocol(format!("Array length {} exceeds maximum {}", len, MAX_ARRAY_LEN))); }
        if buf.remaining() < len { return Err(WaylandServerError::Protocol(format!("Buffer too small for array of length {}", len))); }
        let array_data = buf.copy_to_bytes(len);
        let padding = (4 - (len % 4)) % 4;
        if buf.remaining() < padding { return Err(WaylandServerError::Protocol("Buffer too small for array padding".to_string())); }
        buf.advance(padding); Ok(array_data)
    }
    pub fn read_fd(buf: &mut Bytes, fds_for_current_message: &mut Vec<RawFd>) -> Result<RawFd, WaylandServerError> {
        let _placeholder_fd_val = Self::read_u32(buf)?;
        if fds_for_current_message.is_empty() {
            return Err(WaylandServerError::Protocol("Expected file descriptor, but none were available".to_string()));
        }
        Ok(fds_for_current_message.remove(0))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    fn create_test_header_bytes(object_id: u32, size: u16, opcode: u16) -> Bytes {
        let mut buf = BytesMut::with_capacity(MESSAGE_HEADER_SIZE);
        buf.extend_from_slice(&object_id.to_le_bytes());
        let size_opcode = ((size as u32) << 16) | (opcode as u32);
        buf.extend_from_slice(&size_opcode.to_le_bytes());
        buf.freeze()
    }

    #[test]
    fn test_message_header_parse_success() {
        let mut data = create_test_header_bytes(10, 12, 1);
        let header = MessageHeader::parse(&mut data).unwrap();
        assert_eq!(header.object_id.value(), 10);
    }

    #[test]
    fn test_message_parser_fragmented_message_from_step10() { // Renamed to avoid potential conflict if a similar name exists
        let mut parser = MessageParser::new();
        // Message: obj_id=2, size=16, opcode=1, payload=two u32s (8 bytes)
        let mut full_msg = BytesMut::new();
        full_msg.extend_from_slice(&2u32.to_le_bytes()); // obj_id
        full_msg.extend_from_slice(&((16u32 << 16) | 1u32).to_le_bytes()); // size_opcode
        full_msg.extend_from_slice(&111u32.to_le_bytes()); // arg1
        full_msg.extend_from_slice(&222u32.to_le_bytes()); // arg2

        // Fragment 1: Header only
        parser.append_data(&full_msg.slice(0..MESSAGE_HEADER_SIZE), vec![]);
        assert!(parser.next_message().unwrap().is_none(), "Should not parse with header only");

        // Fragment 2: Part of payload
        parser.append_data(&full_msg.slice(MESSAGE_HEADER_SIZE..MESSAGE_HEADER_SIZE + 4), vec![]);
        assert!(parser.next_message().unwrap().is_none(), "Should not parse with partial payload");

        // Fragment 3: Rest of payload
        parser.append_data(&full_msg.slice(MESSAGE_HEADER_SIZE + 4..), vec![100, 101]); // Add some FDs

        let raw_message_opt = parser.next_message().unwrap();
        assert!(raw_message_opt.is_some(), "Should parse complete message now");
        let raw_message = raw_message_opt.unwrap();

        assert_eq!(raw_message.header.object_id.value(), 2);
        assert_eq!(raw_message.header.size, 16);
        assert_eq!(raw_message.header.opcode, 1);
        assert_eq!(raw_message.content.len(), 8);
        assert_eq!(raw_message.fds, vec![100, 101]);

        let mut content_buf = raw_message.content.clone();
        assert_eq!(MessageParser::read_u32(&mut content_buf).unwrap(), 111);
        assert_eq!(MessageParser::read_u32(&mut content_buf).unwrap(), 222);

        assert!(parser.next_message().unwrap().is_none(), "Should be no more messages");
    }

    #[test]
    fn test_read_fixed_point() {
        let mut data = Bytes::from_static(&256i32.to_le_bytes());
        assert_eq!(MessageParser::read_fixed(&mut data).unwrap(), 1.0);

        let mut data_neg = Bytes::from_static(&(-640i32).to_le_bytes());
        assert_eq!(MessageParser::read_fixed(&mut data_neg).unwrap(), -2.5);

        let mut data_half = Bytes::from_static(&128i32.to_le_bytes());
        assert_eq!(MessageParser::read_fixed(&mut data_half).unwrap(), 0.5);
    }

    #[test]
    fn test_read_string_simple_from_step10() { // Renamed
        let mut buf = BytesMut::new();
        let test_str = "hello";
        let len_with_null = test_str.len() + 1;
        buf.extend_from_slice(&(len_with_null as u32).to_le_bytes());
        buf.extend_from_slice(test_str.as_bytes());
        buf.put_u8(0);
        buf.put_u8(0); buf.put_u8(0);

        let mut data = buf.freeze();
        let s = MessageParser::read_string(&mut data).unwrap();
        assert_eq!(s, "hello");
        assert_eq!(data.remaining(), 0);
    }

    #[test]
    fn test_read_string_exact_multiple_of_4_len_from_step10() { // Renamed
        let mut buf = BytesMut::new();
        let test_str = "bye";
        let len_with_null = test_str.len() + 1;
        buf.extend_from_slice(&(len_with_null as u32).to_le_bytes());
        buf.extend_from_slice(test_str.as_bytes());
        buf.put_u8(0);

        let mut data = buf.freeze();
        let s = MessageParser::read_string(&mut data).unwrap();
        assert_eq!(s, "bye");
        assert_eq!(data.remaining(), 0);
    }

    #[test]
    fn test_read_string_invalid_utf8_from_step10() { // Renamed
        let mut buf = BytesMut::new();
        let len_with_null = 4u32;
        buf.extend_from_slice(&len_with_null.to_le_bytes());
        buf.extend_from_slice(&[b'h', b'i', 0xff]);
        buf.put_u8(0);

        let mut data = buf.freeze();
        let result = MessageParser::read_string(&mut data);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_max_len_exact_from_step10() { // Renamed
        let mut buf = BytesMut::new();
        let len = 4096u32; // MAX_STRING_LEN
        buf.extend_from_slice(&len.to_le_bytes());
        let test_str_bytes = vec![b'a'; (len - 1) as usize];
        buf.extend_from_slice(&test_str_bytes);
        buf.put_u8(0);
        let mut data = buf.freeze();
        let result = MessageParser::read_string(&mut data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_max_len_exceeded_from_step10() { // Renamed
        let mut buf = BytesMut::new();
        let len = 4097u32;
        buf.extend_from_slice(&len.to_le_bytes());
        let mut data = buf.freeze();
        let result = MessageParser::read_string(&mut data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_new_id_from_step10() { // Renamed
        let mut data = Bytes::from_static(&1234u32.to_le_bytes());
        let new_id = MessageParser::read_new_id(&mut data).unwrap();
        assert_eq!(new_id.value(), 1234);
        assert_eq!(data.remaining(), 0);
    }

    #[test]
    fn test_read_array_simple_from_step10() { // Renamed
        let mut buf = BytesMut::new();
        let array_content: [u8; 5] = [1, 2, 3, 4, 5];
        let len = array_content.len() as u32;
        buf.extend_from_slice(&len.to_le_bytes());
        buf.extend_from_slice(&array_content);
        buf.put_u8(0); buf.put_u8(0); buf.put_u8(0);

        let mut data = buf.freeze();
        let result_array = MessageParser::read_array(&mut data).unwrap();
        assert_eq!(result_array.as_ref(), &array_content);
        assert_eq!(data.remaining(), 0);
    }

    #[test]
    fn test_read_array_empty_from_step10() { // Renamed
        let mut buf = BytesMut::new();
        let len = 0u32;
        buf.extend_from_slice(&len.to_le_bytes());

        let mut data = buf.freeze();
        let result_array = MessageParser::read_array(&mut data).unwrap();
        assert!(result_array.is_empty());
        assert_eq!(data.remaining(), 0);
    }

    #[test]
    fn test_read_array_exact_multiple_of_4_len_from_step10() { // Renamed
        let mut buf = BytesMut::new();
        let array_content: [u8; 4] = [10, 20, 30, 40];
        let len = array_content.len() as u32;
        buf.extend_from_slice(&len.to_le_bytes());
        buf.extend_from_slice(&array_content);

        let mut data = buf.freeze();
        let result_array = MessageParser::read_array(&mut data).unwrap();
        assert_eq!(result_array.as_ref(), &array_content);
        assert_eq!(data.remaining(), 0);
    }

    #[test]
    fn test_read_array_too_large_from_step10() { // Renamed
        let mut buf = BytesMut::new();
        let len = (1024 * 1024 * 2) as u32;
        buf.extend_from_slice(&len.to_le_bytes());
        let mut data = buf.freeze();
        let result = MessageParser::read_array(&mut data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_fd_success_from_step10() { // Renamed
        let mut data = Bytes::from_static(&0u32.to_le_bytes());
        let mut fds = vec![10, 20];

        let fd1 = MessageParser::read_fd(&mut data, &mut fds).unwrap();
        assert_eq!(fd1, 10);
        assert_eq!(data.remaining(), 0);
        assert_eq!(fds.len(), 1);

        let mut data2 = Bytes::from_static(&1u32.to_le_bytes());
        let fd2 = MessageParser::read_fd(&mut data2, &mut fds).unwrap();
        assert_eq!(fd2, 20);
        assert_eq!(data2.remaining(), 0);
        assert!(fds.is_empty());
    }

    #[test]
    fn test_read_fd_no_available_fds_from_step10() { // Renamed
        let mut data = Bytes::from_static(&0u32.to_le_bytes());
        let mut fds = vec![];

        let result = MessageParser::read_fd(&mut data, &mut fds);
        assert!(result.is_err());
    }
    // End of restored tests from step 10

    fn get_mock_sig(name: &str, opcode: u16) -> MessageSignature {
        mock_spec_store().get(&(name.to_string(), opcode)).cloned().unwrap()
    }

    #[test]
    fn test_validate_wl_surface_attach_correct() {
        let sig = get_mock_sig("wl_surface", 0);
        let mut content = BytesMut::new();
        content.extend_from_slice(&100u32.to_le_bytes());
        content.extend_from_slice(&10i32.to_le_bytes());
        content.extend_from_slice(&20i32.to_le_bytes());

        let raw_msg = RawMessage {
            header: MessageHeader { object_id: ObjectId(1), size: (MESSAGE_HEADER_SIZE + 12) as u16, opcode: 0 },
            content: content.freeze(),
            fds: vec![],
        };

        let args = MessageParser::validate_and_parse_args(&raw_msg, &sig).unwrap();
        assert_eq!(args.len(), 3);
        match args[0] { ArgumentValue::Object(id) => assert_eq!(id.value(), 100), _ => panic!("Wrong type") }
        match args[1] { ArgumentValue::Int(val) => assert_eq!(val, 10), _ => panic!("Wrong type") }
        match args[2] { ArgumentValue::Int(val) => assert_eq!(val, 20), _ => panic!("Wrong type") }
    }

    #[test]
    fn test_validate_wl_surface_attach_too_few_args_in_buffer() {
        let sig = get_mock_sig("wl_surface", 0);
        let mut content = BytesMut::new();
        content.extend_from_slice(&100u32.to_le_bytes());
        content.extend_from_slice(&10i32.to_le_bytes());

        let raw_msg = RawMessage {
            header: MessageHeader { object_id: ObjectId(1), size: (MESSAGE_HEADER_SIZE + 8) as u16, opcode: 0 },
            content: content.freeze(),
            fds: vec![],
        };

        let result = MessageParser::validate_and_parse_args(&raw_msg, &sig);
        assert!(result.is_err());
        if let Err(WaylandServerError::Protocol(msg)) = result {
            assert!(msg.contains("Buffer too small for i32"));
        } else {
            panic!("Expected protocol error");
        }
    }

    #[test]
    fn test_validate_wl_surface_attach_too_many_args_in_buffer() {
        let sig = get_mock_sig("wl_surface", 0);
        let mut content = BytesMut::new();
        content.extend_from_slice(&100u32.to_le_bytes());
        content.extend_from_slice(&10i32.to_le_bytes());
        content.extend_from_slice(&20i32.to_le_bytes());
        content.extend_from_slice(&0u32.to_le_bytes());

        let raw_msg = RawMessage {
            header: MessageHeader { object_id: ObjectId(1), size: (MESSAGE_HEADER_SIZE + 16) as u16, opcode: 0 },
            content: content.freeze(),
            fds: vec![],
        };

        let result = MessageParser::validate_and_parse_args(&raw_msg, &sig);
        assert!(result.is_err());
        if let Err(WaylandServerError::Protocol(msg)) = result {
            assert!(msg.contains("Too much data for message"));
            assert!(msg.contains("4 unexpected leftover bytes"));
        } else {
            panic!("Expected protocol error for leftover data");
        }
    }

    #[test]
    fn test_validate_shm_pool_create_buffer_correct_with_fd() {
        let sig = get_mock_sig("wl_shm_pool", 0);
        let mut content = BytesMut::new();
        content.extend_from_slice(&3001u32.to_le_bytes());
        content.extend_from_slice(&0u32.to_le_bytes());
        content.extend_from_slice(&0i32.to_le_bytes());
        content.extend_from_slice(&640i32.to_le_bytes());
        content.extend_from_slice(&480i32.to_le_bytes());
        content.extend_from_slice(&1280i32.to_le_bytes());
        content.extend_from_slice(&1u32.to_le_bytes());

        let mock_fd = 7;
        let raw_msg = RawMessage {
            header: MessageHeader { object_id: ObjectId(100), size: (MESSAGE_HEADER_SIZE + 4*7) as u16, opcode: 0 },
            content: content.freeze(),
            fds: vec![mock_fd],
        };

        let args = MessageParser::validate_and_parse_args(&raw_msg, &sig).unwrap();
        assert_eq!(args.len(), 7);
        match args[0] { ArgumentValue::NewId(id) => assert_eq!(id.value(), 3001), _ => panic!("Wrong type for new_id") }
        match args[1] { ArgumentValue::Fd(fd) => assert_eq!(fd, mock_fd), _ => panic!("Wrong type for fd") }
        match args[2] { ArgumentValue::Int(val) => assert_eq!(val, 0), _ => panic!("Wrong type for offset") }
        match args[6] { ArgumentValue::Uint(val) => assert_eq!(val, 1), _ => panic!("Wrong type for format") }
    }

    #[test]
    fn test_validate_shm_pool_create_buffer_missing_fd_in_vec() {
        let sig = get_mock_sig("wl_shm_pool", 0);
        let mut content = BytesMut::new();
        content.extend_from_slice(&3001u32.to_le_bytes());
        content.extend_from_slice(&0u32.to_le_bytes());
        for _ in 0..5 { content.extend_from_slice(&0i32.to_le_bytes()); }
        content.extend_from_slice(&0u32.to_le_bytes());

        let raw_msg = RawMessage {
            header: MessageHeader { object_id: ObjectId(100), size: (MESSAGE_HEADER_SIZE + 4*7) as u16, opcode: 0 },
            content: content.freeze(),
            fds: vec![],
        };

        let result = MessageParser::validate_and_parse_args(&raw_msg, &sig);
        assert!(result.is_err());
        if let Err(WaylandServerError::Protocol(msg)) = result {
            assert!(msg.contains("Expected file descriptor, but none were available"));
        } else {
            panic!("Expected protocol error for missing FD in RawMessage.fds");
        }
    }

    #[test]
    fn test_validate_shm_pool_create_buffer_too_many_fds_in_vec() {
        let sig = get_mock_sig("wl_shm_pool", 0);
        let mut content = BytesMut::new();
        content.extend_from_slice(&3001u32.to_le_bytes());
        content.extend_from_slice(&0u32.to_le_bytes());
        for _ in 0..5 { content.extend_from_slice(&0i32.to_le_bytes()); }
        content.extend_from_slice(&0u32.to_le_bytes());

        let raw_msg = RawMessage {
            header: MessageHeader { object_id: ObjectId(100), size: (MESSAGE_HEADER_SIZE + 4*7) as u16, opcode: 0 },
            content: content.freeze(),
            fds: vec![7, 8],
        };

        let result = MessageParser::validate_and_parse_args(&raw_msg, &sig);
        assert!(result.is_err());
        if let Err(WaylandServerError::Protocol(msg)) = result {
            assert!(msg.contains("Too many FDs for message"));
            assert!(msg.contains("1 unexpected leftover FDs"));
        } else {
            panic!("Expected protocol error for too many FDs");
        }
    }
}
