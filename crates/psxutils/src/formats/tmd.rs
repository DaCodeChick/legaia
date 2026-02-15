//! TMD (PlayStation Model Data) format parser
//!
//! Parses standard PSX TMD (PlayStation Model Data) format files.
//!
//! ## TMD Format Structure
//!
//! ```text
//! Header (12 bytes):
//!   u32 id            // 0x00000041 (fixed magic number)
//!   u32 flags         // Flags (usually 0)
//!   u32 num_objects   // Number of objects in file
//!
//! Object Table (num_objects * 28 bytes):
//!   For each object:
//!     u32 vert_offset   // Offset to vertex data
//!     u32 vert_count    // Number of vertices
//!     u32 normal_offset // Offset to normal data
//!     u32 normal_count  // Number of normals
//!     u32 prim_offset   // Offset to primitive data
//!     u32 prim_count    // Number of primitives
//!     i32 scale         // Fixed-point scale (usually 1)
//!
//! Vertex/Normal Data: i16 x, y, z + u16 padding (8 bytes each)
//! Primitive Data: Variable format based on primitive type
//! ```

use crate::{PsxError, Result};

/// TMD format magic number
pub const TMD_MAGIC: u32 = 0x00000041;

/// TMD model file
#[derive(Debug, Clone)]
pub struct Tmd {
    /// Flags from header
    pub flags: u32,
    /// Objects (meshes) in this model
    pub objects: Vec<TmdObject>,
}

/// Single object (mesh) in a TMD file
#[derive(Debug, Clone)]
pub struct TmdObject {
    /// Vertices
    pub vertices: Vec<TmdVertex>,
    /// Normals
    pub normals: Vec<TmdNormal>,
    /// Primitives (faces)
    pub primitives: Vec<TmdPrimitive>,
    /// Fixed-point scale factor
    pub scale: i32,
}

/// 3D vertex position
#[derive(Debug, Clone, Copy)]
pub struct TmdVertex {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

/// 3D normal vector
#[derive(Debug, Clone, Copy)]
pub struct TmdNormal {
    pub nx: i16,
    pub ny: i16,
    pub nz: i16,
}

/// Primitive (polygon) in TMD
#[derive(Debug, Clone)]
pub enum TmdPrimitive {
    /// Triangle with 3 vertices
    Triangle {
        /// Vertex indices
        vertices: [u16; 3],
        /// Normal indices (one per vertex for smooth shading)
        normals: Option<[u16; 3]>,
        /// Texture coordinates (if textured)
        uvs: Option<[(u8, u8); 3]>,
        /// Vertex colors (if colored)
        colors: Option<[(u8, u8, u8); 3]>,
        /// Texture page/CLUT info
        texture_info: Option<TextureInfo>,
    },
    /// Quad with 4 vertices
    Quad {
        /// Vertex indices
        vertices: [u16; 4],
        /// Normal indices (one per vertex for smooth shading)
        normals: Option<[u16; 4]>,
        /// Texture coordinates (if textured)
        uvs: Option<[(u8, u8); 4]>,
        /// Vertex colors (if colored)
        colors: Option<[(u8, u8, u8); 4]>,
        /// Texture page/CLUT info
        texture_info: Option<TextureInfo>,
    },
}

/// Texture page and CLUT (Color Lookup Table) information
#[derive(Debug, Clone, Copy)]
pub struct TextureInfo {
    /// CLUT X coordinate (in VRAM)
    pub clut_x: u16,
    /// CLUT Y coordinate (in VRAM)
    pub clut_y: u16,
    /// Texture page info
    pub tpage: u16,
}

impl Tmd {
    /// Parse a TMD file from bytes
    ///
    /// Parses standard PSX TMD format with magic number 0x00000041.
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 12 {
            return Err(PsxError::ParseError(
                "TMD file too small for header".to_string(),
            ));
        }

        // Check magic at offset 0
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if magic != TMD_MAGIC {
            return Err(PsxError::ParseError(format!(
                "Invalid TMD magic number: expected {:#010x}, found {:#010x}",
                TMD_MAGIC, magic
            )));
        }

        Self::parse_standard_tmd(data)
    }

    /// Parse standard PSX TMD format
    fn parse_standard_tmd(data: &[u8]) -> Result<Self> {
        let flags = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let num_objects = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;

        if num_objects == 0 {
            return Err(PsxError::ParseError("TMD has zero objects".to_string()));
        }

        // Sanity check: reject absurdly large object counts
        const MAX_OBJECTS: usize = 1000;
        if num_objects > MAX_OBJECTS {
            return Err(PsxError::ParseError(format!(
                "TMD object count too large: {} (max {})",
                num_objects, MAX_OBJECTS
            )));
        }

        // Parse object table (starts at offset 12)
        let mut objects = Vec::with_capacity(num_objects);
        let obj_table_offset = 12;

        for i in 0..num_objects {
            let obj_offset = obj_table_offset + (i * 28);
            if obj_offset + 28 > data.len() {
                return Err(PsxError::ParseError(format!(
                    "Object table entry {} out of bounds",
                    i
                )));
            }

            let obj_data = &data[obj_offset..obj_offset + 28];
            let object = Self::parse_object(data, obj_data)?;
            objects.push(object);
        }

        Ok(Self { flags, objects })
    }

    /// Parse a single object from the object table entry
    fn parse_object(file_data: &[u8], obj_entry: &[u8]) -> Result<TmdObject> {
        // Read object table entry
        let vert_offset =
            u32::from_le_bytes([obj_entry[0], obj_entry[1], obj_entry[2], obj_entry[3]]) as usize;
        let vert_count =
            u32::from_le_bytes([obj_entry[4], obj_entry[5], obj_entry[6], obj_entry[7]]) as usize;
        let normal_offset =
            u32::from_le_bytes([obj_entry[8], obj_entry[9], obj_entry[10], obj_entry[11]]) as usize;
        let normal_count =
            u32::from_le_bytes([obj_entry[12], obj_entry[13], obj_entry[14], obj_entry[15]])
                as usize;
        let _prim_offset =
            u32::from_le_bytes([obj_entry[16], obj_entry[17], obj_entry[18], obj_entry[19]])
                as usize;
        let _prim_count =
            u32::from_le_bytes([obj_entry[20], obj_entry[21], obj_entry[22], obj_entry[23]])
                as usize;
        let scale =
            i32::from_le_bytes([obj_entry[24], obj_entry[25], obj_entry[26], obj_entry[27]]);

        // Sanity checks for vertex/normal counts
        const MAX_VERTS: usize = 100000;
        const MAX_NORMALS: usize = 100000;

        if vert_count > MAX_VERTS {
            return Err(PsxError::ParseError(format!(
                "TMD vertex count too large: {} (max {})",
                vert_count, MAX_VERTS
            )));
        }

        if normal_count > MAX_NORMALS {
            return Err(PsxError::ParseError(format!(
                "TMD normal count too large: {} (max {})",
                normal_count, MAX_NORMALS
            )));
        }

        // Parse vertices
        let mut vertices = Vec::with_capacity(vert_count);
        for i in 0..vert_count {
            let voffset = vert_offset + (i * 8);
            if voffset + 8 > file_data.len() {
                return Err(PsxError::ParseError(format!("Vertex {} out of bounds", i)));
            }

            let vdata = &file_data[voffset..voffset + 8];
            vertices.push(TmdVertex {
                x: i16::from_le_bytes([vdata[0], vdata[1]]),
                y: i16::from_le_bytes([vdata[2], vdata[3]]),
                z: i16::from_le_bytes([vdata[4], vdata[5]]),
            });
        }

        // Parse normals
        let mut normals = Vec::with_capacity(normal_count);
        for i in 0..normal_count {
            let noffset = normal_offset + (i * 8);
            if noffset + 8 > file_data.len() {
                return Err(PsxError::ParseError(format!("Normal {} out of bounds", i)));
            }

            let ndata = &file_data[noffset..noffset + 8];
            normals.push(TmdNormal {
                nx: i16::from_le_bytes([ndata[0], ndata[1]]),
                ny: i16::from_le_bytes([ndata[2], ndata[3]]),
                nz: i16::from_le_bytes([ndata[4], ndata[5]]),
            });
        }

        // Parse primitives
        let mut primitives = Vec::new();
        let mut prim_pos = _prim_offset;

        for _ in 0.._prim_count {
            if prim_pos >= file_data.len() {
                break; // Reached end of data
            }

            // Parse primitive header
            if prim_pos + 4 > file_data.len() {
                break;
            }

            let prim = Self::parse_primitive(file_data, prim_pos)?;
            let packet_size = Self::primitive_packet_size(file_data, prim_pos)?;

            primitives.push(prim);
            prim_pos += packet_size;
        }

        Ok(TmdObject {
            vertices,
            normals,
            primitives,
            scale,
        })
    }

    /// Parse a single primitive from data
    fn parse_primitive(data: &[u8], offset: usize) -> Result<TmdPrimitive> {
        if offset + 4 > data.len() {
            return Err(PsxError::ParseError(
                "Primitive header out of bounds".to_string(),
            ));
        }

        // Primitive packet structure:
        // Byte 0: olen (packet length in 32-bit words)
        // Byte 1: ilen (packet header length in 32-bit words)
        // Byte 2: flag (primitive flags)
        // Byte 3: mode (primitive mode/type)

        let _olen = data[offset] as usize;
        let _ilen = data[offset + 1] as usize;
        let flag = data[offset + 2];
        let mode = data[offset + 3];

        // Determine primitive type from mode/flag
        let is_quad = (mode & 0x08) != 0;
        let is_textured = (mode & 0x04) != 0;
        let is_gouraud = (mode & 0x10) != 0; // Smooth shading (per-vertex normals)
        let _is_light_calc = (flag & 0x01) == 0; // Light source calculation enabled

        let pos = offset + 4;

        if is_quad {
            Self::parse_quad(data, pos, is_textured, is_gouraud)
        } else {
            Self::parse_triangle(data, pos, is_textured, is_gouraud)
        }
    }

    /// Parse a triangle primitive
    fn parse_triangle(
        data: &[u8],
        mut pos: usize,
        is_textured: bool,
        is_gouraud: bool,
    ) -> Result<TmdPrimitive> {
        // Normal indices (0 or 3 depending on gouraud)
        let normals = if is_gouraud {
            if pos + 6 > data.len() {
                return Err(PsxError::ParseError(
                    "Triangle normals out of bounds".to_string(),
                ));
            }
            let n0 = u16::from_le_bytes([data[pos], data[pos + 1]]);
            let n1 = u16::from_le_bytes([data[pos + 2], data[pos + 3]]);
            let n2 = u16::from_le_bytes([data[pos + 4], data[pos + 5]]);
            pos += 6;
            Some([n0, n1, n2])
        } else {
            if pos + 2 > data.len() {
                return Err(PsxError::ParseError(
                    "Triangle normal out of bounds".to_string(),
                ));
            }
            let n = u16::from_le_bytes([data[pos], data[pos + 1]]);
            pos += 2;
            Some([n, n, n]) // Use same normal for all vertices
        };

        // Vertex indices
        if pos + 6 > data.len() {
            return Err(PsxError::ParseError(
                "Triangle vertices out of bounds".to_string(),
            ));
        }
        let v0 = u16::from_le_bytes([data[pos], data[pos + 1]]);
        let v1 = u16::from_le_bytes([data[pos + 2], data[pos + 3]]);
        let v2 = u16::from_le_bytes([data[pos + 4], data[pos + 5]]);
        pos += 6;

        // UVs and texture info (if textured)
        let (uvs, texture_info) = if is_textured {
            if pos + 12 > data.len() {
                return Err(PsxError::ParseError(
                    "Triangle texture data out of bounds".to_string(),
                ));
            }

            let u0 = data[pos];
            let v0_uv = data[pos + 1];
            let clut_x = u16::from_le_bytes([data[pos + 2], data[pos + 3]]);
            let clut_y = u16::from_le_bytes([data[pos + 4], data[pos + 5]]);

            let u1 = data[pos + 6];
            let v1_uv = data[pos + 7];
            let tpage = u16::from_le_bytes([data[pos + 8], data[pos + 9]]);

            let u2 = data[pos + 10];
            let v2_uv = data[pos + 11];

            (
                Some([(u0, v0_uv), (u1, v1_uv), (u2, v2_uv)]),
                Some(TextureInfo {
                    clut_x,
                    clut_y,
                    tpage,
                }),
            )
        } else {
            (None, None)
        };

        Ok(TmdPrimitive::Triangle {
            vertices: [v0, v1, v2],
            normals,
            uvs,
            colors: None, // Colors typically not stored in TMD
            texture_info,
        })
    }

    /// Parse a quad primitive
    fn parse_quad(
        data: &[u8],
        mut pos: usize,
        is_textured: bool,
        is_gouraud: bool,
    ) -> Result<TmdPrimitive> {
        // Normal indices (0 or 4 depending on gouraud)
        let normals = if is_gouraud {
            if pos + 8 > data.len() {
                return Err(PsxError::ParseError(
                    "Quad normals out of bounds".to_string(),
                ));
            }
            let n0 = u16::from_le_bytes([data[pos], data[pos + 1]]);
            let n1 = u16::from_le_bytes([data[pos + 2], data[pos + 3]]);
            let n2 = u16::from_le_bytes([data[pos + 4], data[pos + 5]]);
            let n3 = u16::from_le_bytes([data[pos + 6], data[pos + 7]]);
            pos += 8;
            Some([n0, n1, n2, n3])
        } else {
            if pos + 2 > data.len() {
                return Err(PsxError::ParseError(
                    "Quad normal out of bounds".to_string(),
                ));
            }
            let n = u16::from_le_bytes([data[pos], data[pos + 1]]);
            pos += 2;
            Some([n, n, n, n]) // Use same normal for all vertices
        };

        // Vertex indices
        if pos + 8 > data.len() {
            return Err(PsxError::ParseError(
                "Quad vertices out of bounds".to_string(),
            ));
        }
        let v0 = u16::from_le_bytes([data[pos], data[pos + 1]]);
        let v1 = u16::from_le_bytes([data[pos + 2], data[pos + 3]]);
        let v2 = u16::from_le_bytes([data[pos + 4], data[pos + 5]]);
        let v3 = u16::from_le_bytes([data[pos + 6], data[pos + 7]]);
        pos += 8;

        // UVs and texture info (if textured)
        let (uvs, texture_info) = if is_textured {
            if pos + 16 > data.len() {
                return Err(PsxError::ParseError(
                    "Quad texture data out of bounds".to_string(),
                ));
            }

            let u0 = data[pos];
            let v0_uv = data[pos + 1];
            let clut_x = u16::from_le_bytes([data[pos + 2], data[pos + 3]]);
            let clut_y = u16::from_le_bytes([data[pos + 4], data[pos + 5]]);

            let u1 = data[pos + 6];
            let v1_uv = data[pos + 7];
            let tpage = u16::from_le_bytes([data[pos + 8], data[pos + 9]]);

            let u2 = data[pos + 10];
            let v2_uv = data[pos + 11];

            let u3 = data[pos + 14];
            let v3_uv = data[pos + 15];

            (
                Some([(u0, v0_uv), (u1, v1_uv), (u2, v2_uv), (u3, v3_uv)]),
                Some(TextureInfo {
                    clut_x,
                    clut_y,
                    tpage,
                }),
            )
        } else {
            (None, None)
        };

        Ok(TmdPrimitive::Quad {
            vertices: [v0, v1, v2, v3],
            normals,
            uvs,
            colors: None,
            texture_info,
        })
    }

    /// Calculate the size of a primitive packet in bytes
    fn primitive_packet_size(data: &[u8], offset: usize) -> Result<usize> {
        if offset >= data.len() {
            return Err(PsxError::ParseError(
                "Primitive offset out of bounds".to_string(),
            ));
        }

        // olen is the packet length in 32-bit words (including the header)
        let olen = data[offset] as usize;

        // Convert from 32-bit words to bytes
        Ok(olen * 4)
    }

    /// Convert to normalized floating point vertices
    ///
    /// Converts 16-bit signed integer coordinates to normalized f32 coordinates
    pub fn to_f32_vertices(&self) -> Vec<Vec<[f32; 3]>> {
        self.objects
            .iter()
            .map(|obj| {
                let scale = if obj.scale == 0 {
                    1.0
                } else {
                    obj.scale as f32
                };

                obj.vertices
                    .iter()
                    .map(|v| [v.x as f32 / scale, v.y as f32 / scale, v.z as f32 / scale])
                    .collect()
            })
            .collect()
    }

    /// Convert to normalized floating point normals
    pub fn to_f32_normals(&self) -> Vec<Vec<[f32; 3]>> {
        self.objects
            .iter()
            .map(|obj| {
                obj.normals
                    .iter()
                    .map(|n| {
                        // Normalize the normal vector
                        let nx = n.nx as f32 / 4096.0;
                        let ny = n.ny as f32 / 4096.0;
                        let nz = n.nz as f32 / 4096.0;
                        let len = (nx * nx + ny * ny + nz * nz).sqrt();
                        if len > 0.0 {
                            [nx / len, ny / len, nz / len]
                        } else {
                            [0.0, 1.0, 0.0] // Default up vector
                        }
                    })
                    .collect()
            })
            .collect()
    }

    /// Get the number of objects
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    /// Get vertex count for a specific object
    pub fn vertex_count(&self, object_index: usize) -> Option<usize> {
        self.objects.get(object_index).map(|obj| obj.vertices.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tmd_magic() {
        assert_eq!(TMD_MAGIC, 0x00000041);
    }

    #[test]
    fn test_tmd_parse_empty() {
        let data = vec![0; 11];
        assert!(Tmd::parse(&data).is_err());
    }

    #[test]
    fn test_tmd_parse_invalid_magic() {
        let mut data = vec![0; 12];
        // Set wrong magic
        data[0..4].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
        assert!(Tmd::parse(&data).is_err());
    }
}
