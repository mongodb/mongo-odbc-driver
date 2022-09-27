#[non_exhaustive]
pub struct BsonTypeInfo {
    pub type_name: &'static str,
    pub searchable: bool,
    pub scale: Option<u16>,
    pub precision: Option<u16>,
    pub octet_length: Option<u16>,
    pub fixed_bytes_length: Option<u16>,
}

impl BsonTypeInfo {
    pub const DOUBLE: BsonTypeInfo = BsonTypeInfo {
        type_name: "double",
        searchable: true,
        scale: Some(15),
        precision: Some(15),
        octet_length: None,
        fixed_bytes_length: Some(8),
    };
    pub const STRING: BsonTypeInfo = BsonTypeInfo {
        type_name: "string",
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const OBJECT: BsonTypeInfo = BsonTypeInfo {
        type_name: "object",
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const ARRAY: BsonTypeInfo = BsonTypeInfo {
        type_name: "array",
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const BINDATA: BsonTypeInfo = BsonTypeInfo {
        type_name: "binData",
        searchable: false,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const UNDEFINED: BsonTypeInfo = BsonTypeInfo {
        type_name: "undefined",
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const OBJECTID: BsonTypeInfo = BsonTypeInfo {
        type_name: "objectId",
        searchable: true,
        scale: None,
        precision: Some(24),
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const BOOL: BsonTypeInfo = BsonTypeInfo {
        type_name: "bool",
        searchable: true,
        scale: None,
        precision: Some(1),
        octet_length: None,
        fixed_bytes_length: Some(1),
    };
    pub const DATE: BsonTypeInfo = BsonTypeInfo {
        type_name: "date",
        searchable: true,
        scale: Some(3),
        precision: Some(24),
        octet_length: None,
        fixed_bytes_length: Some(8),
    };
    pub const NULL: BsonTypeInfo = BsonTypeInfo {
        type_name: "null",
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const REGEX: BsonTypeInfo = BsonTypeInfo {
        type_name: "regex",
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const DBPOINTER: BsonTypeInfo = BsonTypeInfo {
        type_name: "dbPointer",
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const JAVASCRIPT: BsonTypeInfo = BsonTypeInfo {
        type_name: "javascript",
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const SYMBOL: BsonTypeInfo = BsonTypeInfo {
        type_name: "symbol",
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const JAVASCRIPTWITHSCOPE: BsonTypeInfo = BsonTypeInfo {
        type_name: "javascriptWithScope",
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const INT: BsonTypeInfo = BsonTypeInfo {
        type_name: "int",
        searchable: true,
        scale: Some(0),
        precision: Some(10),
        octet_length: None,
        fixed_bytes_length: Some(4),
    };
    pub const TIMESTAMP: BsonTypeInfo = BsonTypeInfo {
        type_name: "timestamp",
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const LONG: BsonTypeInfo = BsonTypeInfo {
        type_name: "long",
        searchable: true,
        scale: Some(0),
        precision: Some(19),
        octet_length: None,
        fixed_bytes_length: Some(8),
    };
    pub const DECIMAL: BsonTypeInfo = BsonTypeInfo {
        type_name: "decimal",
        searchable: true,
        scale: Some(34),
        precision: Some(34),
        octet_length: None,
        fixed_bytes_length: Some(16),
    };
    pub const MINKEY: BsonTypeInfo = BsonTypeInfo {
        type_name: "minKey",
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const MAXKEY: BsonTypeInfo = BsonTypeInfo {
        type_name: "maxKey",
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
}
