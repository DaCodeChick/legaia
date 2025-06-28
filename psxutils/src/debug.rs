/// The class of a symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum SymbolClass {
	Stack = 1,
	Array,
	StructRef = 8,
	Address = 10,
	TypeAlias = 13,
	StructEnd = 102,
}

pub struct StructSymbol {
	offset: u32,
	class: SymbolClass,
	type_id: u16,
	size: u32,
	array_dimensions: u16,
	array_entries_per_dimension: u32,
	internal_name: String,
	name: String,
}

/// A chunk of debug information for a symbol.
#[derive(Debug, Clone)]
pub enum SymbolChunk {
	/// A function symbol chunk (address)
	Function(u32),
	/// A source code line symbol chunk (address)
	SourceCodeLine(u32),
	/// A source code 8-bit symbol chunk (address, line number)
	SourceCode8Bit(u32, u8),
	/// A source code 16-bit symbol chunk (address, line number)
	SourceCode16Bit(u32, u16),
	/// A source code 32-bit symbol chunk (address, line number)
	SourceCode32Bit(u32, u32),
	/// A source code file symbol chunk (address, first line, file path)
	SourceCodeFile(u32, u32, String),
	/// A source code end symbol chunk (address)
	SourceCodeEnd(u32),
	/// An internal function symbol chunk (address, file path, function name)
	InternalFunction(u32, String, String),
	/// An internal function end symbol chunk (address, line number)
	InternalFunctionEnd(u32, u32),
	/// A primitive type symbol chunk (offset, class, type, size, name)
	PrimitiveType(u32, SymbolClass, u16, u32, String),
	/// A complex type symbol chunk
	ComplexType(StructSymbol),
}

pub struct DebugSymbol {
	address: u32,
	id: u8,
}
