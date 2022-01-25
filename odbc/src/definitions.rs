use num_derive::FromPrimitive;

#[macro_export]
macro_rules! map {
	($($key:expr => $val:expr),* $(,)?) => {
		std::iter::Iterator::collect([
			$({
				($key, $val)
			},)*
		].into_iter())
	};
}

#[derive(Debug)]
pub enum AppRowDescriptor {
    SqlNullDesc = 0,
}

#[derive(Debug)]
pub enum AppParamDescriptor {
    SqlNullDesc = 0,
}

#[derive(Debug)]
pub enum CursorScrollable {
    NonScrollable = 0,
    Scrollable,
}

#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive)]
pub enum CursorSensitivity {
    Unspecified = 0,
    Insensitive,
    Sensitive,
}

#[derive(Debug)]
pub enum AsyncEnable {
    Off = 0,
    On,
}

#[derive(Debug)]
pub enum Concurrency {
    ReadOnly = 0,
    Lock,
    RowVer,
    Values,
}

#[derive(Debug)]
pub enum CursorType {
    ForwardOnly = 0,
    Static,
    KeysetDriven,
    Dynamic, // TODO: make sure the discriminants are correct
}

#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive)]
pub enum NoScan {
    Off = 0,
    On,
}

#[derive(Debug)]
pub enum BindType {
    BindByColumn = 0,
}

#[derive(Debug)]
pub enum ParamOperationPtr {}

#[derive(Debug)]
pub enum ParamStatusPtr {}

#[derive(Debug)]
pub enum ParamsProcessedPtr {}

#[derive(Debug)]
pub enum ParamsetSize {}

#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive)]
pub enum RetrieveData {
    Off = 0,
    On,
}

#[derive(Debug)]
pub enum RowOperationPtr {}

#[derive(Debug)]
pub enum SimulateCursor {
    NonUnique = 0, // TODO: check enum numbers
    TryUnique,
    Unique,
}

#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive)]
pub enum UseBookmarks {
    Off = 0,
    Variable = 2,
}

#[derive(Debug)]
pub enum AsyncStmtEvent {}

#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive)]
pub enum SqlBool {
    SqlFalse = 0,
    SqlTrue = 2,
}
