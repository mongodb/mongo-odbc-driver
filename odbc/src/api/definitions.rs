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

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum SqlBool {
    False = 0,
    True,
}

// Environment attributes

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum OdbcVersion {
    Odbc3 = 3,
    Odbc3_80 = 380,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum ConnectionPooling {
    Off = 0,
    OnePerDriver,
    OnePerHEnv,
    DriverAware,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub enum CpMatch {
    Strict = 0,
    Relaxed,
}

// Statement attributes

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum CursorScrollable {
    NonScrollable = 0,
    Scrollable,
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum CursorSensitivity {
    Unspecified = 0,
    Insensitive,
    Sensitive,
}

#[derive(Clone, Copy, Debug)]
pub enum AsyncEnable {
    Off = 0,
    On,
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum Concurrency {
    ReadOnly = 1,
    Lock = 2,
    RowVer = 4,
    Values = 8,
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum CursorType {
    ForwardOnly = 0,
    KeysetDriven = -1,
    Dynamic = -2,
    Static = -3,
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum NoScan {
    Off = 0,
    On,
}

#[derive(Clone, Copy, Debug)]
pub enum BindType {
    BindByColumn = 0,
}

#[derive(Clone, Copy, Debug)]
pub enum ParamOperationPtr {}

#[derive(Clone, Copy, Debug)]
pub enum ParamStatusPtr {}

#[derive(Clone, Copy, Debug)]
pub enum ParamsProcessedPtr {}

#[derive(Clone, Copy, Debug)]
pub enum ParamsetSize {}

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum RetrieveData {
    Off = 0,
    On,
}

#[derive(Clone, Copy, Debug)]
pub enum RowOperationPtr {}

#[derive(Clone, Copy, Debug)]
pub enum SimulateCursor {
    NonUnique = 0,
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum UseBookmarks {
    Off = 0,
    Variable = 2,
}

#[derive(Clone, Copy, Debug)]
pub enum AsyncStmtEvent {}
