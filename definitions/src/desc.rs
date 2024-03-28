use num_derive::FromPrimitive;

/// Used in `SQLColAttributeW`.
#[allow(non_camel_case_types)]
#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, FromPrimitive)]
pub enum Desc {
    /// `SQL_DESC_COUNT`. Returned in `NumericAttributePtr`. The number of columns available in the
    /// result set. This returns 0 if there are no columns in the result set. The value in the
    /// `column_number` argument is ignored.
    SQL_DESC_COUNT = 1001,
    /// `SQL_DESC_TYPE`. Retruned in `NumericAttributePtr`. A numeric value that specifies the SQL
    /// data type. When ColumnNumber is equal to 0, SQL_BINARY is returned for variable-length
    /// bookmarks and SQL_INTEGER is returned for fixed-length bookmarks. For the datetime and
    /// interval data types, this field returns the verbose data type: SQL_DATETIME or SQL_INTERVAL.
    /// Note: To work against ODBC 2.x drivers, use `SQL_DESC_CONCISE_TYPE` instead.
    SQL_DESC_TYPE = 1002,
    /// `SQL_DESC_LENGTH`. Returned in `NumericAttributePtr`. A numeric value that is either the
    /// maximum or actual character length of a character string or binary data type. It is the
    /// maximum character length for a fixed-length data type, or the actual character length for a
    /// variable-length data type. Its value always excludes the null-termination byte that ends the
    /// character string.
    SQL_DESC_LENGTH = 1003,
    /// `SQL_DESC_OCTET_LENGTH_PTR`.
    SQL_DESC_OCTET_LENGTH_PTR = 1004,
    /// `SQL_DESC_PRECISION`. Returned in `NumericAttributePtr`. A numeric value that for a numeric
    /// data type denotes the applicable precision. For data types SQL_TYPE_TIME,
    /// SQL_TYPE_TIMESTAMP, and all the interval data types that represent a time interval, its
    /// value is the applicable precision of the fractional seconds component.
    SQL_DESC_PRECISION = 1005,
    /// `SQL_DESC_SCALE`. Returned in `NumericAttributePtr`. A numeric value that is the applicable
    /// scale for a numeric data type. For DECIMAL and NUMERIC data types, this is the defined
    /// scale. It is undefined for all other data types.
    SQL_DESC_SCALE = 1006,
    /// `SQL_DESC_DATETIME_INTERVAL_CODE`.
    SQL_DESC_DATETIME_INTERVAL_CODE = 1007,
    /// `SQL_DESC_NULLABLE`. Returned in `NumericAttributePtr`. `SQL_ NULLABLE` if the column can
    /// have NULL values; SQL_NO_NULLS if the column does not have NULL values; or
    /// SQL_NULLABLE_UNKNOWN if it is not known whether the column accepts NULL values.
    SQL_DESC_NULLABLE = 1008,
    /// `SQL_DESC_INDICATOR_PTR`
    SQL_DESC_INDICATOR_PTR = 1009,
    /// `SQL_DESC_DATA_PTR`.
    SQL_DESC_DATA_PTR = 1010,
    /// `SQL_DESC_NAME`. Returned in `CharacterAttributePtr`. The column alias, if it applies. If
    /// the column alias does not apply, the column name is returned. In either case,
    /// SQL_DESC_UNNAMED is set to SQL_NAMED. If there is no column name or a column alias, an empty
    /// string is returned and SQL_DESC_UNNAMED is set to SQL_UNNAMED.
    SQL_DESC_NAME = 1011,
    /// `SQL_DESC_UNNAMED`. Returned in `NumericAttributePtr`. SQL_NAMED or SQL_UNNAMED. If the
    /// SQL_DESC_NAME field of the IRD contains a column alias or a column name, SQL_NAMED is
    /// returned. If there is no column name or column alias, SQL_UNNAMED is returned.
    SQL_DESC_UNNAMED = 1012,
    /// `SQL_DESC_OCTET_LENGTH`. Returned in `NumericAttributePtr`. The length, in bytes, of a
    /// character string or binary data type. For fixed-length character or binary types, this is
    /// the actual length in bytes. For variable-length character or binary types, this is the
    /// maximum length in bytes. This value does not include the null terminator.
    SQL_DESC_OCTET_LENGTH = 1013,
    SQL_DESC_ALLOC_TYPE = 1099,
    #[cfg(feature = "odbc_version_4")]
    SQL_DESC_CHARACTER_SET_CATALOG = 1018,
    #[cfg(feature = "odbc_version_4")]
    SQL_DESC_CHARACTER_SET_SCHEMA = 1019,
    #[cfg(feature = "odbc_version_4")]
    #[cfg(feature = "odbc_version_4")]
    SQL_DESC_CHARACTER_SET_NAME = 1020,
    #[cfg(feature = "odbc_version_4")]
    SQL_DESC_COLLATION_CATALOG = 1015,
    #[cfg(feature = "odbc_version_4")]
    SQL_DESC_COLLATION_SCHEMA = 1016,
    #[cfg(feature = "odbc_version_4")]
    SQL_DESC_COLLATION_NAME = 1017,
    #[cfg(feature = "odbc_version_4")]
    SQL_DESC_USER_DEFINED_TYPE_CATALOG = 1026,
    #[cfg(feature = "odbc_version_4")]
    SQL_DESC_USER_DEFINED_TYPE_SCHEMA = 1027,
    #[cfg(feature = "odbc_version_4")]
    SQL_DESC_USER_DEFINED_TYPE_NAME = 1028,

    SQL_DESC_ARRAY_SIZE = 20,
    SQL_DESC_ARRAY_STATUS_PTR = 21,
    /// `SQL_DESC_AUTO_UNIQUE_VALUE`. Returned in `NumericAttributePtr`. `true` if the column is an
    /// autoincrementing column. `false` if the column is not an autoincrementing column or is not
    /// numeric. This field is valid for numeric data type columns only. An application can insert
    /// values into a row containing an autoincrement column, but typically cannot update values in
    /// the column. When an insert is made into an autoincrement column, a unique value is inserted
    /// into the column at insert time. The increment is not defined, but is data source-specific.
    /// An application should not assume that an autoincrement column starts at any particular point
    /// or increments by any particular value.
    SQL_DESC_AUTO_UNIQUE_VALUE = 11,
    /// `SQL_DESC_BASE_COLUMN_NAME`. Returned in `CharacterAttributePtr`. The base column name for
    /// the result set column. If a base column name does not exist (as in the case of columns that
    /// are expressions), then this variable contains an empty string.
    SQL_DESC_BASE_COLUMN_NAME = 22,
    /// `SQL_DESC_BASE_TABLE_NAME`. Returned in `CharacterAttributePtr`. The name of the base table
    /// that contains the column. If the base table name cannot be defined or is not applicable,
    /// then this variable contains an empty string.
    SQL_DESC_BASE_TABLE_NAME = 23,
    SQL_DESC_BIND_OFFSET_PTR = 24,
    SQL_DESC_BIND_TYPE = 25,
    /// `SQL_DESC_CASE_SENSITIVE`. Returned in `NumericAttributePtr`. `true` if the column is
    /// treated as case-sensitive for collations and comparisons. `false` if the column is not
    /// treated as case-sensitive for collations and comparisons or is noncharacter.
    SQL_DESC_CASE_SENSITIVE = 12,
    /// `SQL_DESC_CATALOG_NAME`. Returned in `CharacterAttributePtr`. The catalog of the table that
    /// contains the column. The returned value is implementation-defined if the column is an
    /// expression or if the column is part of a view. If the data source does not support catalogs
    /// or the catalog name cannot be determined, an empty string is returned. This VARCHAR record
    /// field is not limited to 128 characters.
    SQL_DESC_CATALOG_NAME = 17,
    /// `SQL_DESC_CONCISE_TYPE`. Returned in `NumericAttributePtr`. The concise data type. For the
    /// datetime and interval data types, this field returns the concise data type; for example,
    /// SQL_TYPE_TIME or SQL_INTERVAL_YEAR.
    SQL_DESC_CONCISE_TYPE = 2,
    SQL_DESC_DATETIME_INTERVAL_PRECISION = 26,
    /// `SQL_DESC_DISPLAY_SIZE`. Returned in `NumericAttributePtr`. Maximum number of characters
    /// required to display data from the column.
    SQL_DESC_DISPLAY_SIZE = 6,
    /// `SQL_DESC_FIXED_PREC_SCALE`. Returned in `NumericAttributePtr`. `true` if the column has a
    /// fixed precision and nonzero scale that are data source-specific. `false` if the column does
    /// not have a fixed precision and nonzero scale that are data source-specific.
    SQL_DESC_FIXED_PREC_SCALE = 9,
    /// `SQL_DESC_LABEL`. Returned in `CharacterAttributePtr`. The column label or title. For
    /// example, a column named EmpName might be labeled Employee Name or might be labeled with an
    /// alias. If a column does not have a label, the column name is returned. If the column is
    /// unlabeled and unnamed, an empty string is returned.
    SQL_DESC_LABEL = 18,
    /// `SQL_DESC_LITERAL_PREFIX`. Returned in `CharacterAttributePtr`. This VARCHAR(128) record
    /// field contains the character or characters that the driver recognizes as a prefix for a
    /// literal of this data type. This field contains an empty string for a data type for which a
    /// literal prefix is not applicable.
    SQL_DESC_LITERAL_PREFIX = 27,
    /// `SQL_DESC_LITERAL_SUFFIX`. Returned in `CharacterAttributePtr`. This VARCHAR(128) record
    /// field contains the character or characters that the driver recognizes as a suffix for a
    /// literal of this data type. This field contains an empty string for a data type for which a
    /// literal suffix is not applicable.
    SQL_DESC_LITERAL_SUFFIX = 28,
    /// `SQL_DESC_LOCAL_TYPE_NAME`. Returned in `CharacterAttributePtr`. This VARCHAR(128) record
    /// field contains any localized (native language) name for the data type that may be different
    /// from the regular name of the data type. If there is no localized name, then an empty string
    /// is returned. This field is for display purposes only. The character set of the string is
    /// locale-dependent and is typically the default character set of the server.
    SQL_DESC_LOCAL_TYPE_NAME = 29,
    SQL_DESC_MAXIMUM_SCALE = 30,
    SQL_DESC_MINIMUM_SCALE = 31,
    /// `SQL_DESC_NUM_PREC_RADIX`. Returned in `NumericAttributePtr`. If the data type in the
    /// SQL_DESC_TYPE field is an approximate numeric data type, this SQLINTEGER field contains a
    /// value of 2 because the SQL_DESC_PRECISION field contains the number of bits. If the data
    /// type in the SQL_DESC_TYPE field is an exact numeric data type, this field contains a value
    /// of 10 because the SQL_DESC_PRECISION field contains the number of decimal digits. This field
    /// is set to 0 for all non-numeric data types.
    SQL_DESC_NUM_PREC_RADIX = 32,
    SQL_DESC_PARAMETER_TYPE = 33,
    SQL_DESC_ROWS_PROCESSED_PTR = 34,
    #[cfg(feature = "odbc_version_3_50")]
    SQL_DESC_ROWVER = 35,
    /// `SQL_DESC_SCHEMA_NAME`. Returned in `CharacterAttributePtr`. The schema of the table that
    /// contains the column. The returned value is implementation-defined if the column is an
    /// expression or if the column is part of a view. If the data source does not support schemas
    /// or the schema name cannot be determined, an empty string is returned. This VARCHAR record
    /// field is not limited to 128 characters.
    SQL_DESC_SCHEMA_NAME = 16,
    /// `SQL_DESC_SEARCHABLE`. Returned in `NumericAttributePtr`. `SQL_PRED_NONE` if the column
    /// cannot be used in a WHERE clause. `SQL_PRED_CHAR` if the column can be used in a WHERE
    /// clause but only with the LIKE predicate. `SQL_PRED_BASIC` if the column can be used in a
    /// WHERE clause with all the comparison operators except LIKE. `SQL_PRED_SEARCHABLE` if the
    /// column can be used in a WHERE clause with any comparison operator. Columns of type
    /// `SQL_LONGVARCHAR` and `SQL_LONGVARBINARY` usually return `SQL_PRED_CHAR`.
    SQL_DESC_SEARCHABLE = 13,
    /// `SQL_DESC_TYPE_NAME`. Returned in `CharacterAttributePtr`. Data source-dependent data type
    /// name; for example, "CHAR", "VARCHAR", "MONEY", "LONG VARBINARY", or "CHAR ( ) FOR BIT DATA".
    /// If the type is unknown, an empty string is returned.
    SQL_DESC_TYPE_NAME = 14,
    /// `SQL_DESC_TABLE_NAME`. Returned in `CharacterAttributePtr`. The name of the table that
    /// contains the column. The returned value is implementation-defined if the column is an
    /// expression or if the column is part of a view. If the table name can not be determined an
    /// empty string is returned.
    SQL_DESC_TABLE_NAME = 15,
    /// `SQL_DESC_UNSIGNED`. Returned in `NumericAttributePtr`. `true` if the column is unsigned (or
    /// not numeric). `false` if the column is signed.
    SQL_DESC_UNSIGNED = 8,
    /// `SQL_DESC_UPDATABLE`. Returned in `NumericAttributePtr`. Column is described by the values
    /// for the defined constants: `SQL_ATTR_READONLY`, `SQL_ATTR_WRITE`,
    /// `SQL_ATTR_READWRITE_UNKNOWN`.
    /// `Updatable` Describes the updatability of the column in the result set, not the column in
    /// the base table. The updatability of the base column on which the result set column is based
    /// may be different from the value in this field. Whether a column is updatable can be based on
    /// the data type, user privileges, and the definition of the result set itself. If it is
    /// unclear whether a column is updatable, `SQL_ATTR_READWRITE_UNKNOWN` should be returned.
    SQL_DESC_UPDATABLE = 10,
    #[cfg(feature = "odbc_version_4")]
    SQL_DESC_MIME_TYPE = 36,
}

/// Used in `SQLColAttributeW`.
#[allow(non_camel_case_types)]
#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AllocType {
    SQL_DESC_ALLOC_AUTO = 1,
    SQL_DESC_ALLOC_USER = 2,
}
