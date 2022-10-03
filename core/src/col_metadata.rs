use crate::{json_schema::simplified::Schema, BsonTypeInfo};
use odbc_sys::SqlDataType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColumnNullability {
    Nullable,
    NoNulls,
    Unknown,
}

// Metadata information for a column of the result set.
// The information is to be used when reporting columns information from
// SQLColAttribute or SQLDescribeCol and when converting the data to the targeted C type.
#[derive(Clone, Debug)]
pub struct MongoColMetadata {
    pub base_col_name: String,
    pub base_table_name: String,
    pub catalog_name: String,
    pub display_size: Option<u16>,
    pub fixed_prec_scale: bool,
    pub label: String,
    pub length: Option<u16>,
    pub col_name: String,
    pub is_nullable: ColumnNullability,
    pub octet_length: Option<u16>,
    pub precision: Option<u16>,
    pub scale: Option<u16>,
    pub is_searchable: bool,
    pub table_name: String,
    // BSON type name
    pub type_name: String,
    // Sql type integer
    pub sql_type: SqlDataType,
    pub is_unsigned: bool,
    pub is_updatable: bool,
}

impl MongoColMetadata {
    pub fn new(
        _current_db: &str,
        datasource_name: String,
        field_name: String,
        field_schema: Schema,
        is_nullable: ColumnNullability,
    ) -> MongoColMetadata {
        let bson_type_info: BsonTypeInfo = (&field_schema).into();
        let sql_type: SqlDataType = (&field_schema).into();

        MongoColMetadata {
            // For base_col_name, base_table_name, and catalog_name, we do
            // not have this information in sqlGetResultSchema, so these will
            // always be empty string for now.
            base_col_name: "".to_string(),
            base_table_name: "".to_string(),
            catalog_name: "".to_string(),
            display_size: bson_type_info.fixed_bytes_length,
            fixed_prec_scale: false,
            label: field_name.clone(),
            length: bson_type_info.fixed_bytes_length,
            col_name: field_name,
            is_nullable,
            octet_length: bson_type_info.octet_length,
            precision: bson_type_info.precision,
            scale: bson_type_info.scale,
            is_searchable: bson_type_info.searchable,
            table_name: datasource_name,
            type_name: bson_type_info.type_name.to_string(),
            sql_type,
            is_unsigned: false,
            is_updatable: false,
        }
    }
}
