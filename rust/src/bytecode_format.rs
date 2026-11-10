//! Platform-independent, experimental Nobody bytecode file format (v2).
//! Decoder still supports legacy v1 files with untyped function metadata.
//!
//! Fixed-width little-endian values and strict bounds make it independent of
//! Rust's struct layout, pointer width and host endianness. This format is
//! EXPERIMENTAL and not promised to remain compatible across releases.
//! FNV-1a detects accidental corruption; it is NOT a signature or security
//! boundary. Never execute untrusted files outside an OS sandbox.
use crate::bytecode::{
    verify, Bytecode, CompiledFunction, Instruction, ValueType, BYTECODE_VERSION,
};
use crate::Diagnostic;

const MAGIC: &[u8; 8] = b"NOBBC001";
const WIRE_VERSION: u16 = 2;
const TEXT_WIRE_VERSION: u16 = 3;
const LEGACY_WIRE_VERSION: u16 = 1;
const MAX_UNITS: usize = 1024;
const MAX_OPCODES: usize = 10_000;
const MAX_NAME_BYTES: usize = 256;
pub const MAX_FILE_BYTES: usize = 2 * 1024 * 1024;

fn invalid(message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        code: 4004,
        line: 0,
        column: 0,
        detail: message.into(),
        repair: "Regenerate this experimental .nbc file with a compatible Nobody compiler; do not run malformed or untrusted bytecode.",
    }
}

fn too_large() -> Diagnostic {
    Diagnostic {
        code: 4005,
        line: 0,
        column: 0,
        detail: "Bytecode file or section exceeds the experimental format budget".into(),
        repair: "Reduce code size or split the program into smaller modules.",
    }
}

// Noncryptographic integrity check, never for authenticity or trust.
fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

fn as_u32(value: usize) -> Result<u32, Diagnostic> {
    u32::try_from(value).map_err(|_| too_large())
}

fn write_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(out: &mut Vec<u8>, value: usize) -> Result<(), Diagnostic> {
    out.extend_from_slice(&as_u32(value)?.to_le_bytes());
    Ok(())
}

fn write_instruction(out: &mut Vec<u8>, op: &Instruction) -> Result<(), Diagnostic> {
    use Instruction::*;
    match op {
        Push(value) => {
            out.push(0);
            out.extend_from_slice(&value.to_le_bytes());
        }
        PushBool(false) => out.extend_from_slice(&[1, 0]),
        PushBool(true) => out.extend_from_slice(&[1, 1]),
        PushNull => out.push(2),
        PushText(value) => {
            out.push(26);
            write_u32(out, value.len())?;
            out.extend_from_slice(value.as_bytes());
        }
        Load(slot) => {
            out.push(3);
            write_u32(out, *slot)?;
        }
        Define(slot) => {
            out.push(4);
            write_u32(out, *slot)?;
        }
        Store(slot) => {
            out.push(5);
            write_u32(out, *slot)?;
        }
        Add => out.push(6),
        Sub => out.push(7),
        Mul => out.push(8),
        Neg => out.push(9),
        Not => out.push(10),
        Eq => out.push(11),
        Ne => out.push(12),
        Lt => out.push(13),
        Le => out.push(14),
        Gt => out.push(15),
        Ge => out.push(16),
        Jump(target) => {
            out.push(17);
            write_u32(out, *target)?;
        }
        JumpIfFalse(target) => {
            out.push(18);
            write_u32(out, *target)?;
        }
        RangeCheck {
            current,
            stop,
            step,
            exit,
        } => {
            out.push(19);
            for field in [current, stop, step, exit] {
                write_u32(out, *field)?;
            }
        }
        RangeAdvance {
            current,
            step,
            exit,
        } => {
            out.push(20);
            for field in [current, step, exit] {
                write_u32(out, *field)?;
            }
        }
        Drop => out.push(21),
        Print(count) => {
            out.push(22);
            write_u32(out, *count)?;
        }
        Call { function, argc } => {
            out.push(23);
            write_u32(out, *function)?;
            write_u32(out, *argc)?;
        }
        Return => out.push(24),
        Halt => out.push(25),
    }
    if out.len() > MAX_FILE_BYTES {
        return Err(too_large());
    }
    Ok(())
}

/// Stable v2 wire tags: 0 = untyped, 1 = int, 2 = bool.
fn write_type(out: &mut Vec<u8>, kind: Option<ValueType>) {
    out.push(match kind {
        None => 0,
        Some(ValueType::Int) => 1,
        Some(ValueType::Bool) => 2,
        Some(ValueType::Str) => 3,
    });
}

fn write_code(out: &mut Vec<u8>, code: &[Instruction]) -> Result<(), Diagnostic> {
    if code.len() > MAX_OPCODES {
        return Err(too_large());
    }
    for instruction in code {
        write_instruction(out, instruction)?;
    }
    Ok(())
}

/// Encode verified in-memory bytecode to self-contained, deterministic bytes.
pub fn encode(program: &Bytecode) -> Result<Vec<u8>, Diagnostic> {
    verify(program)?;
    if program.functions.len() > MAX_UNITS {
        return Err(too_large());
    }
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    // Keep existing v2 bytes stable for integer/boolean-only programs.
    // Text opcodes and typed string signatures require experimental v3.
    let text_program = program
        .code
        .iter()
        .any(|op| matches!(op, Instruction::PushText(_)))
        || program.functions.iter().any(|function| {
            function
                .code
                .iter()
                .any(|op| matches!(op, Instruction::PushText(_)))
                || function.return_type == Some(ValueType::Str)
                || function.parameter_types.contains(&Some(ValueType::Str))
        });
    let version = if text_program {
        TEXT_WIRE_VERSION
    } else {
        WIRE_VERSION
    };
    write_u16(&mut out, version);
    write_u16(&mut out, 0); // reserved flags: reject unknown flags on read
    write_u32(&mut out, program.slots)?;
    write_u32(&mut out, program.code.len())?;
    write_u32(&mut out, program.functions.len())?;
    write_code(&mut out, &program.code)?;
    for function in &program.functions {
        let name = function.name.as_bytes();
        if name.is_empty() || name.len() > MAX_NAME_BYTES {
            return Err(invalid(
                "Function name byte length is outside the file format limits",
            ));
        }
        write_u16(&mut out, name.len() as u16);
        out.extend_from_slice(name);
        write_u32(&mut out, function.arity)?;
        write_u32(&mut out, function.slots)?;
        write_u32(&mut out, function.code.len())?;
        // v2 stores exactly arity one-byte type tags plus one return tag.
        for annotation in &function.parameter_types {
            write_type(&mut out, *annotation);
        }
        write_type(&mut out, function.return_type);
        write_code(&mut out, &function.code)?;
        if out.len() > MAX_FILE_BYTES - 8 {
            return Err(too_large());
        }
    }
    if out.len() > MAX_FILE_BYTES - 8 {
        return Err(too_large());
    }
    let digest = checksum(&out);
    out.extend_from_slice(&digest.to_le_bytes());
    Ok(out)
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
    version: u16,
}

impl<'a> Reader<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8], Diagnostic> {
        let end = self
            .position
            .checked_add(length)
            .ok_or_else(|| invalid("Bytecode offset overflow"))?;
        let part = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| invalid("Truncated bytecode file"))?;
        self.position = end;
        Ok(part)
    }
    fn u8(&mut self) -> Result<u8, Diagnostic> {
        Ok(self.take(1)?[0])
    }
    fn annotation(&mut self) -> Result<Option<ValueType>, Diagnostic> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(ValueType::Int)),
            2 => Ok(Some(ValueType::Bool)),
            3 if self.version == TEXT_WIRE_VERSION => Ok(Some(ValueType::Str)),
            _ => Err(invalid("Invalid bytecode v2 function type tag")),
        }
    }
    fn u16(&mut self) -> Result<u16, Diagnostic> {
        let mut buf = [0u8; 2];
        buf.copy_from_slice(self.take(2)?);
        Ok(u16::from_le_bytes(buf))
    }
    fn u32(&mut self) -> Result<usize, Diagnostic> {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(self.take(4)?);
        Ok(u32::from_le_bytes(buf) as usize)
    }
    fn i128(&mut self) -> Result<i128, Diagnostic> {
        let mut buf = [0u8; 16];
        buf.copy_from_slice(self.take(16)?);
        Ok(i128::from_le_bytes(buf))
    }
    fn opcode(&mut self) -> Result<Instruction, Diagnostic> {
        use Instruction::*;
        match self.u8()? {
            0 => Ok(Push(self.i128()?)),
            1 => match self.u8()? {
                0 => Ok(PushBool(false)),
                1 => Ok(PushBool(true)),
                _ => Err(invalid("Boolean opcode payload must be 0 or 1")),
            },
            2 => Ok(PushNull),
            3 => Ok(Load(self.u32()?)),
            4 => Ok(Define(self.u32()?)),
            5 => Ok(Store(self.u32()?)),
            6 => Ok(Add),
            7 => Ok(Sub),
            8 => Ok(Mul),
            9 => Ok(Neg),
            10 => Ok(Not),
            11 => Ok(Eq),
            12 => Ok(Ne),
            13 => Ok(Lt),
            14 => Ok(Le),
            15 => Ok(Gt),
            16 => Ok(Ge),
            17 => Ok(Jump(self.u32()?)),
            18 => Ok(JumpIfFalse(self.u32()?)),
            19 => Ok(RangeCheck {
                current: self.u32()?,
                stop: self.u32()?,
                step: self.u32()?,
                exit: self.u32()?,
            }),
            20 => Ok(RangeAdvance {
                current: self.u32()?,
                step: self.u32()?,
                exit: self.u32()?,
            }),
            21 => Ok(Drop),
            22 => Ok(Print(self.u32()?)),
            23 => Ok(Call {
                function: self.u32()?,
                argc: self.u32()?,
            }),
            24 => Ok(Return),
            25 => Ok(Halt),
            26 if self.version == TEXT_WIRE_VERSION => {
                let size = self.u32()?;
                if size > 1_048_576 {
                    return Err(too_large());
                }
                let bytes = self.take(size)?;
                let value = std::str::from_utf8(bytes)
                    .map_err(|_| invalid("Bytecode text payload is not valid UTF-8"))?;
                Ok(PushText(value.to_owned()))
            }
            unknown => Err(invalid(format!("Unknown bytecode opcode {unknown}"))),
        }
    }
    fn code(&mut self, count: usize) -> Result<Vec<Instruction>, Diagnostic> {
        if count == 0 || count > MAX_OPCODES {
            return Err(invalid("Invalid bytecode unit instruction count"));
        }
        let mut instructions = Vec::with_capacity(count);
        for _ in 0..count {
            instructions.push(self.opcode()?);
        }
        Ok(instructions)
    }
}

/// Decode with bounded allocations, checksum, exact framing and full VM
/// verification. A correct checksum does not make a file safe or authentic.
pub fn decode(bytes: &[u8]) -> Result<Bytecode, Diagnostic> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err(too_large());
    }
    if bytes.len() < 32 {
        return Err(invalid(
            "Bytecode file is shorter than its header and checksum",
        ));
    }
    let split = bytes.len() - 8;
    let mut hash_bytes = [0u8; 8];
    hash_bytes.copy_from_slice(&bytes[split..]);
    if u64::from_le_bytes(hash_bytes) != checksum(&bytes[..split]) {
        return Err(invalid("Bytecode checksum does not match file contents"));
    }
    let mut reader = Reader {
        bytes: &bytes[..split],
        position: 0,
        version: 0,
    };
    if reader.take(8)? != MAGIC {
        return Err(invalid("Unknown Nobody bytecode file magic"));
    }
    let version = reader.u16()?;
    if version != WIRE_VERSION && version != LEGACY_WIRE_VERSION && version != TEXT_WIRE_VERSION {
        return Err(invalid("Unsupported Nobody bytecode file version"));
    }
    reader.version = version;
    if reader.u16()? != 0 {
        return Err(invalid("Unsupported reserved bytecode flags"));
    }
    let slots = reader.u32()?;
    let opcodes = reader.u32()?;
    let function_count = reader.u32()?;
    if slots > 1024 || function_count > MAX_UNITS {
        return Err(too_large());
    }
    let code = reader.code(opcodes)?;
    let mut functions = Vec::with_capacity(function_count);
    for _ in 0..function_count {
        let name_len = reader.u16()? as usize;
        if name_len == 0 || name_len > MAX_NAME_BYTES {
            return Err(invalid("Invalid bytecode function name size"));
        }
        let name = std::str::from_utf8(reader.take(name_len)?)
            .map_err(|_| invalid("Bytecode function names must be UTF-8"))?
            .to_owned();
        let arity = reader.u32()?;
        let local_slots = reader.u32()?;
        let instruction_count = reader.u32()?;
        if arity > local_slots || local_slots > 1024 {
            return Err(invalid("Invalid bytecode function parameter or slot count"));
        }
        let (parameter_types, return_type) = if version != LEGACY_WIRE_VERSION {
            let mut types = Vec::with_capacity(arity);
            for _ in 0..arity {
                types.push(reader.annotation()?);
            }
            (types, reader.annotation()?)
        } else {
            // v1 has no signatures; it is intentionally treated as untyped.
            (vec![None; arity], None)
        };
        let unit = reader.code(instruction_count)?;
        functions.push(CompiledFunction {
            name,
            arity,
            slots: local_slots,
            parameter_types,
            return_type,
            code: unit,
        });
    }
    if reader.position != reader.bytes.len() {
        return Err(invalid("Bytecode file has unrecognized trailing data"));
    }
    let program = Bytecode {
        version: BYTECODE_VERSION,
        slots,
        code,
        functions,
    };
    verify(&program)?;
    Ok(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::{compile_source, execute};

    fn examples() -> [(&'static str, &'static str); 4] {
        [
            ("ai", include_str!("../examples/bytecode_functions_ai.nbd")),
            ("sk", include_str!("../examples/bytecode_for_sk.nbd")),
            ("en", include_str!("../examples/bytecode_control_en.nbd")),
            (
                "ai",
                include_str!("../examples/bytecode_function_null_ai.nbd"),
            ),
        ]
    }

    fn refresh_checksum(bytes: &mut [u8]) {
        let split = bytes.len() - 8;
        let hash = checksum(&bytes[..split]);
        bytes[split..].copy_from_slice(&hash.to_le_bytes());
    }

    #[test]
    fn text_v3_roundtrip_is_portable_and_backwards_compatible() {
        let example = "fn message(s: str): str { return s + \" ☸\" }\nprint(message(\"Život\"), \"\" == \"\")";
        let original = compile_source(example, "ai").unwrap();
        let bytes = encode(&original).unwrap();
        assert_eq!(u16::from_le_bytes([bytes[8], bytes[9]]), TEXT_WIRE_VERSION);
        let decoded = decode(&bytes).unwrap();
        assert_eq!(original, decoded);
        assert_eq!(
            execute(&decoded).unwrap(),
            crate::evaluate_source(example, "ai").unwrap()
        );
        assert_eq!(encode(&decoded).unwrap(), bytes);
        let old = compile_source("print(2 + 3)", "ai").unwrap();
        let old_bytes = encode(&old).unwrap();
        assert_eq!(
            u16::from_le_bytes([old_bytes[8], old_bytes[9]]),
            WIRE_VERSION
        );
    }

    #[test]
    fn v3_text_payload_rejects_invalid_utf8_without_execution() {
        let original = compile_source("print(\"ab\")", "ai").unwrap();
        let mut bytes = encode(&original).unwrap();
        // Header is 24 bytes, then push-text opcode(26), size(4), payload.
        let first = bytes.iter().position(|value| *value == 26).unwrap();
        let payload_start = first + 5;
        bytes[payload_start] = 0xff;
        refresh_checksum(&mut bytes);
        assert_eq!(decode(&bytes).unwrap_err().code, 4004);
    }

    #[test]
    fn exact_reproducible_roundtrips_with_functions_and_loops() {
        for (profile, source) in examples() {
            let original = compile_source(source, profile).unwrap();
            let encoded = encode(&original).unwrap();
            let restored = decode(&encoded).unwrap();
            assert_eq!(restored, original);
            assert_eq!(encode(&restored).unwrap(), encoded);
            assert_eq!(execute(&restored).unwrap(), execute(&original).unwrap());
        }
    }

    #[test]
    fn same_canonical_ir_across_languages_has_identical_bytes() {
        let ai =
            compile_source(include_str!("../examples/bytecode_functions_ai.nbd"), "ai").unwrap();
        let sk =
            compile_source(include_str!("../examples/bytecode_functions_sk.nbd"), "sk").unwrap();
        let en =
            compile_source(include_str!("../examples/bytecode_functions_en.nbd"), "en").unwrap();
        assert_eq!(encode(&ai).unwrap(), encode(&sk).unwrap());
        assert_eq!(encode(&ai).unwrap(), encode(&en).unwrap());
    }

    fn legacy_v1_bytes(program: &Bytecode) -> Vec<u8> {
        verify(program).unwrap();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(MAGIC);
        write_u16(&mut bytes, LEGACY_WIRE_VERSION);
        write_u16(&mut bytes, 0);
        write_u32(&mut bytes, program.slots).unwrap();
        write_u32(&mut bytes, program.code.len()).unwrap();
        write_u32(&mut bytes, program.functions.len()).unwrap();
        write_code(&mut bytes, &program.code).unwrap();
        for function in &program.functions {
            assert!(function.parameter_types.iter().all(Option::is_none));
            assert_eq!(function.return_type, None);
            write_u16(&mut bytes, function.name.len() as u16);
            bytes.extend_from_slice(function.name.as_bytes());
            write_u32(&mut bytes, function.arity).unwrap();
            write_u32(&mut bytes, function.slots).unwrap();
            write_u32(&mut bytes, function.code.len()).unwrap();
            write_code(&mut bytes, &function.code).unwrap();
        }
        let hash = checksum(&bytes);
        bytes.extend_from_slice(&hash.to_le_bytes());
        bytes
    }

    #[test]
    fn v2_signatures_roundtrip_and_match_three_syntax_profiles() {
        let sources = [
            (
                "ai",
                include_str!("../examples/native_typed_signatures_ai.nbd"),
            ),
            (
                "sk",
                include_str!("../examples/native_typed_signatures_sk.nbd"),
            ),
            (
                "en",
                include_str!("../examples/native_typed_signatures_en.nbd"),
            ),
        ];
        let mut binaries = Vec::new();
        for (profile, source) in sources {
            let compiled = compile_source(source, profile).unwrap();
            assert_eq!(
                compiled.functions[0].parameter_types,
                vec![
                    Some(ValueType::Bool),
                    Some(ValueType::Int),
                    Some(ValueType::Int)
                ]
            );
            assert_eq!(compiled.functions[0].return_type, Some(ValueType::Int));
            let bytes = encode(&compiled).unwrap();
            assert_eq!(&bytes[8..10], &WIRE_VERSION.to_le_bytes());
            let restored = decode(&bytes).unwrap();
            assert_eq!(restored, compiled);
            assert_eq!(execute(&restored).unwrap(), "3\n8\nfalse\n5\n");
            assert_eq!(
                execute(&restored).unwrap(),
                crate::evaluate_source(source, profile).unwrap()
            );
            binaries.push(bytes);
        }
        assert_eq!(binaries[0], binaries[1]);
        assert_eq!(binaries[0], binaries[2]);
    }

    #[test]
    fn legacy_v1_function_files_are_readable_as_untyped() {
        let source = include_str!("../examples/bytecode_functions_ai.nbd");
        let original = compile_source(source, "ai").unwrap();
        let legacy = legacy_v1_bytes(&original);
        assert_eq!(&legacy[8..10], &LEGACY_WIRE_VERSION.to_le_bytes());
        let restored = decode(&legacy).unwrap();
        assert_eq!(restored, original);
        assert_eq!(execute(&restored).unwrap(), "13\n6\n");
        // Reading v1 and emitting v2 preserves meaning while adding explicit
        // untyped metadata, rather than changing the old file in place.
        let upgraded = encode(&restored).unwrap();
        assert_eq!(&upgraded[8..10], &WIRE_VERSION.to_le_bytes());
        assert_eq!(decode(&upgraded).unwrap(), original);
    }

    #[test]
    fn malformed_v2_type_tag_fails_even_with_valid_checksum() {
        let compiled = compile_source(
            "fn identity(flag: bool): bool { return flag }\nprint(identity(true))",
            "ai",
        )
        .unwrap();
        let mut bytes = encode(&compiled).unwrap();
        let split = bytes.len() - 8;
        let mut reader = Reader {
            bytes: &bytes[..split],
            position: 0,
        };
        reader.take(8).unwrap();
        reader.u16().unwrap();
        reader.u16().unwrap();
        reader.u32().unwrap();
        let main_code = reader.u32().unwrap();
        assert_eq!(reader.u32().unwrap(), 1);
        reader.code(main_code).unwrap();
        let name_length = reader.u16().unwrap() as usize;
        reader.take(name_length).unwrap();
        assert_eq!(reader.u32().unwrap(), 1);
        reader.u32().unwrap();
        reader.u32().unwrap();
        let first_parameter_tag = reader.position;
        bytes[first_parameter_tag] = 0xFF;
        refresh_checksum(&mut bytes);
        assert_eq!(decode(&bytes).unwrap_err().code, 4004);
        bytes[first_parameter_tag] = 2;
        bytes[first_parameter_tag + 1] = 0xFF;
        refresh_checksum(&mut bytes);
        assert_eq!(decode(&bytes).unwrap_err().code, 4004);
        // Extra tags must never be silently accepted.
        bytes[first_parameter_tag] = 2;
        bytes[first_parameter_tag + 1] = 2;
        bytes.insert(first_parameter_tag + 2, 0);
        refresh_checksum(&mut bytes);
        assert_eq!(decode(&bytes).unwrap_err().code, 4004);
    }

    #[test]
    fn corruption_unknown_versions_and_truncation_fail_closed() {
        let source = "print(9)";
        let compiled = compile_source(source, "ai").unwrap();
        let valid = encode(&compiled).unwrap();
        assert_eq!(&valid[0..8], MAGIC);
        assert_eq!(decode(&valid).unwrap(), compiled);
        let mut corrupt = valid.clone();
        corrupt[24] ^= 0x80;
        assert_eq!(decode(&corrupt).unwrap_err().code, 4004);
        refresh_checksum(&mut corrupt);
        assert_eq!(decode(&corrupt).unwrap_err().code, 4004);

        let mut wrong_version = valid.clone();
        wrong_version[8] = 99;
        refresh_checksum(&mut wrong_version);
        assert_eq!(decode(&wrong_version).unwrap_err().code, 4004);
        let mut bad_flags = valid.clone();
        bad_flags[10] = 1;
        refresh_checksum(&mut bad_flags);
        assert_eq!(decode(&bad_flags).unwrap_err().code, 4004);
        for cut in [0usize, 1, 10, 24, valid.len() - 1] {
            assert!(decode(&valid[..cut]).is_err());
        }
        let mut extra = valid.clone();
        extra.insert(extra.len() - 8, 0);
        refresh_checksum(&mut extra);
        assert_eq!(decode(&extra).unwrap_err().code, 4004);
    }

    #[test]
    fn verifier_runs_even_if_corrupted_jump_has_valid_checksum() {
        let compiled = compile_source("print(2)", "ai").unwrap();
        let mut bytes = encode(&compiled).unwrap();
        // First opcode is Push i128 (17 bytes); next is Print(1).
        // Replace the one-byte Print tag with Jump(1), retaining its u32 arg:
        // jump back to the middle of its predecessor (a stack error).
        bytes[24 + 17] = 17;
        refresh_checksum(&mut bytes);
        assert_eq!(decode(&bytes).unwrap_err().code, 4004);
    }

    #[test]
    fn oversized_and_short_frames_are_rejected_before_allocating_units() {
        assert_eq!(
            decode(&vec![0u8; MAX_FILE_BYTES + 1]).unwrap_err().code,
            4005
        );
        let compiled = compile_source("print(1)", "ai").unwrap();
        let mut bytes = encode(&compiled).unwrap();
        bytes[16..20].copy_from_slice(&u32::MAX.to_le_bytes());
        refresh_checksum(&mut bytes);
        assert!(decode(&bytes).is_err());
    }
}
