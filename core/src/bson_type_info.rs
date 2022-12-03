use odbc_sys::SqlDataType;

#[non_exhaustive]
pub struct BsonTypeInfo {
    pub type_name: &'static str,
    pub sql_type: SqlDataType,
    pub searchable: bool,
    pub scale: Option<u16>,
    pub precision: Option<u16>,
    pub octet_length: Option<u16>,
    pub fixed_bytes_length: Option<u16>,
}

impl BsonTypeInfo {
    pub const DOUBLE: BsonTypeInfo = BsonTypeInfo {
        type_name: "double",
        sql_type: SqlDataType::DOUBLE,
        searchable: true,
        scale: Some(15),
        precision: Some(15),
        octet_length: Some(8),
        fixed_bytes_length: Some(8),
    };
    pub const STRING: BsonTypeInfo = BsonTypeInfo {
        type_name: "string",
        sql_type: SqlDataType::VARCHAR,
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const OBJECT: BsonTypeInfo = BsonTypeInfo {
        type_name: "object",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const ARRAY: BsonTypeInfo = BsonTypeInfo {
        type_name: "array",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const BINDATA: BsonTypeInfo = BsonTypeInfo {
        type_name: "binData",
        sql_type: SqlDataType::EXT_BINARY,
        searchable: false,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const UNDEFINED: BsonTypeInfo = BsonTypeInfo {
        type_name: "undefined",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const OBJECTID: BsonTypeInfo = BsonTypeInfo {
        type_name: "objectId",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        searchable: true,
        scale: None,
        precision: Some(24),
        octet_length: Some(24),
        fixed_bytes_length: None,
    };
    pub const BOOL: BsonTypeInfo = BsonTypeInfo {
        type_name: "bool",
        sql_type: SqlDataType::EXT_BIT,
        searchable: true,
        scale: None,
        precision: Some(1),
        octet_length: Some(1),
        fixed_bytes_length: Some(1),
    };
    pub const DATE: BsonTypeInfo = BsonTypeInfo {
        type_name: "date",
        sql_type: SqlDataType::TIMESTAMP,
        searchable: true,
        scale: Some(3),
        precision: Some(23),
        octet_length: Some(16),
        fixed_bytes_length: Some(16),
    };
    pub const NULL: BsonTypeInfo = BsonTypeInfo {
        type_name: "null",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const REGEX: BsonTypeInfo = BsonTypeInfo {
        type_name: "regex",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const DBPOINTER: BsonTypeInfo = BsonTypeInfo {
        type_name: "dbPointer",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const JAVASCRIPT: BsonTypeInfo = BsonTypeInfo {
        type_name: "javascript",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const SYMBOL: BsonTypeInfo = BsonTypeInfo {
        type_name: "symbol",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const JAVASCRIPTWITHSCOPE: BsonTypeInfo = BsonTypeInfo {
        type_name: "javascriptWithScope",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const INT: BsonTypeInfo = BsonTypeInfo {
        type_name: "int",
        sql_type: SqlDataType::INTEGER,
        searchable: true,
        scale: Some(0),
        precision: Some(10),
        octet_length: Some(4),
        fixed_bytes_length: Some(4),
    };
    pub const TIMESTAMP: BsonTypeInfo = BsonTypeInfo {
        type_name: "timestamp",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const LONG: BsonTypeInfo = BsonTypeInfo {
        type_name: "long",
        sql_type: SqlDataType::EXT_BIG_INT,
        searchable: true,
        scale: Some(0),
        precision: Some(19),
        octet_length: Some(8),
        fixed_bytes_length: Some(8),
    };
    pub const DECIMAL: BsonTypeInfo = BsonTypeInfo {
        type_name: "decimal",
        // TODO SQL-1068: Change to SqlDataType::DECIMAL
        sql_type: SqlDataType::UNKNOWN_TYPE,
        searchable: true,
        scale: None,
        precision: None,          // TODO SQL-1068: Some(34),
        octet_length: None,       // TODO SQL-1068: Some(16),
        fixed_bytes_length: None, // TODO SQL-1068: Some(16),
    };
    pub const MINKEY: BsonTypeInfo = BsonTypeInfo {
        type_name: "minKey",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const MAXKEY: BsonTypeInfo = BsonTypeInfo {
        type_name: "maxKey",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
    pub const BSON: BsonTypeInfo = BsonTypeInfo {
        type_name: "bson",
        sql_type: SqlDataType::UNKNOWN_TYPE,
        searchable: true,
        scale: None,
        precision: None,
        octet_length: None,
        fixed_bytes_length: None,
    };
}
