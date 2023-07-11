mod simple_type_info;
mod standard_type_info;

pub use simple_type_info::SimpleTypeInfo;
pub use standard_type_info::StandardTypeInfo;

#[derive(PartialEq, Debug, Clone)]
pub enum SchemaMode {
    Standard,
    Simple,
}

#[derive(PartialEq, Debug, Clone)]
pub enum BsonTypeInfo{
    Standard(StandardTypeInfo),
    Simple(SimpleTypeInfo),
}
