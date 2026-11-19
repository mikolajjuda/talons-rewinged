use std::collections::HashMap;
use std::fmt;

const TYPE_MASK: u8 = 0x7F;

const SPT_END: u8 = 0;
const SPT_COLOR: u8 = 1;
const SPT_DITHER_COLOR: u8 = 2;
const SPT_THICK: u8 = 3;
const SPT_PLANAR_SYMMETRY: u8 = 4;
const SPT_TRANSFORM_ON: u8 = 5;
const SPT_TRANSFORM_OFF: u8 = 6;
const SPT_SHIFT: u8 = 7;
const SPT_PT: u8 = 8;
const SPT_POLYPT: u8 = 9;
const SPT_LINE: u8 = 10;
const SPT_POLYLINE: u8 = 11;
const SPT_RECT: u8 = 12;
const SPT_ROTATED_RECT: u8 = 13;
const SPT_CIRCLE: u8 = 14;
const SPT_ELLIPSE: u8 = 15;
const SPT_POLYGON: u8 = 16;
const SPT_BSPLINE2: u8 = 17;
const SPT_BSPLINE2_CLOSED: u8 = 18;
const SPT_BSPLINE3: u8 = 19;
const SPT_BSPLINE3_CLOSED: u8 = 20;
const SPT_FLOOD_FILL: u8 = 21;
const SPT_FLOOD_FILL_NOT: u8 = 22;
const SPT_BITMAP: u8 = 23;
const SPT_MESH: u8 = 24;
const SPT_SHIFTABLE_MESH: u8 = 25;
const SPT_ARROW: u8 = 26;
const SPT_TEXT: u8 = 27;
const SPT_TEXT_BOX: u8 = 28;
const SPT_TEXT_DIAMOND: u8 = 29;

const MAX_ELEMENT_COUNT: usize = 1_000_000;

// -----------------------------------------------------------------------------
// Public sprite data structures
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct Sprite {
    pub elements: Vec<SpriteElement>,
    pub bytes_consumed: usize,
    pub trailing_bytes: usize,
}

#[derive(Debug)]
pub struct SpriteElement {
    pub type_code: u8,
    pub selected: bool,
    pub data: SpriteElementData,
}

#[derive(Debug)]
pub enum SpriteElementData {
    Color {
        color: u8,
    },
    DitherColor {
        color: u16,
    },
    Thick {
        thickness: i32,
    },
    PlanarSymmetry {
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    },
    TransformOn,
    TransformOff,
    Shift {
        x: i32,
        y: i32,
    },
    Point {
        x: i32,
        y: i32,
    },
    PolyPoint {
        count: i32,
        x: i32,
        y: i32,
        packed_data: Vec<u8>,
    },
    Line {
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    },
    Polyline {
        points: Vec<(i32, i32)>,
    },
    Rect {
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    },
    RotatedRect {
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        angle: f64,
    },
    Circle {
        x: i32,
        y: i32,
        radius: i32,
    },
    Ellipse {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        angle: f64,
    },
    Polygon {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        angle: f64,
        sides: i32,
    },
    BSpline {
        count: i32,
        points: Vec<(i32, i32, i32)>,
        dimensions: u8,
        closed: bool,
    },
    FloodFill {
        x: i32,
        y: i32,
    },
    FloodFillNot {
        x: i32,
        y: i32,
    },
    Bitmap {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        row_stride: usize,
        pixels: Vec<u8>,
    },
    Mesh {
        vertices: Vec<(i32, i32, i32)>,
        triangles: Vec<MeshTriangle>,
    },
    ShiftableMesh {
        x: i32,
        y: i32,
        z: i32,
        vertices: Vec<(i32, i32, i32)>,
        triangles: Vec<MeshTriangle>,
    },
    Arrow {
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    },
    Text {
        x: i32,
        y: i32,
        text: String,
    },
    TextBox {
        x: i32,
        y: i32,
        text: String,
    },
    TextDiamond {
        x: i32,
        y: i32,
        text: String,
    },
}

#[derive(Debug)]
pub struct MeshTriangle {
    pub color: i32,
    pub indices: [i32; 3],
}

#[derive(Debug)]
pub struct SpriteParseError {
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for SpriteParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Sprite parse error at {:#x}: {}",
            self.offset, self.message
        )
    }
}

impl std::error::Error for SpriteParseError {}

type ParseResult<T> = Result<T, SpriteParseError>;

// -----------------------------------------------------------------------------
// DolDoc / Talons file error
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct TalonsParseError {
    pub offset: usize,
    pub resource_id: Option<u32>,
    pub message: String,
}

impl fmt::Display for TalonsParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(id) = self.resource_id {
            write!(
                f,
                "Talons.HC parse error at {:#x} (BI={id}): {}",
                self.offset, self.message
            )
        } else {
            write!(
                f,
                "Talons.HC parse error at {:#x}: {}",
                self.offset, self.message
            )
        }
    }
}

impl std::error::Error for TalonsParseError {}

// -----------------------------------------------------------------------------
// Binary reader
//
// TempleOS CSprite structures are packed. Do not insert alignment padding
// between the type byte and the following fields.
// -----------------------------------------------------------------------------

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn error<T>(&self, message: impl Into<String>) -> ParseResult<T> {
        Err(SpriteParseError {
            offset: self.pos,
            message: message.into(),
        })
    }

    fn require(&self, count: usize) -> ParseResult<()> {
        if count > self.data.len().saturating_sub(self.pos) {
            return self.error(format!(
                "Need {count} bytes, only {} remain",
                self.data.len().saturating_sub(self.pos)
            ));
        }

        Ok(())
    }

    fn read_u8(&mut self) -> ParseResult<u8> {
        self.require(1)?;
        let value = self.data[self.pos];
        self.pos += 1;
        Ok(value)
    }

    fn read_u16(&mut self) -> ParseResult<u16> {
        self.require(2)?;
        let value = u16::from_le_bytes(self.data[self.pos..self.pos + 2].try_into().unwrap());
        self.pos += 2;
        Ok(value)
    }

    fn read_i32(&mut self) -> ParseResult<i32> {
        self.require(4)?;
        let value = i32::from_le_bytes(self.data[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Ok(value)
    }

    fn read_f64(&mut self) -> ParseResult<f64> {
        self.require(8)?;
        let value = f64::from_le_bytes(self.data[self.pos..self.pos + 8].try_into().unwrap());
        self.pos += 8;
        Ok(value)
    }

    fn read_bytes(&mut self, count: usize) -> ParseResult<Vec<u8>> {
        self.require(count)?;
        let result = self.data[self.pos..self.pos + count].to_vec();
        self.pos += count;
        Ok(result)
    }

    fn read_point2(&mut self) -> ParseResult<(i32, i32)> {
        Ok((self.read_i32()?, self.read_i32()?))
    }

    fn read_point3(&mut self) -> ParseResult<(i32, i32, i32)> {
        Ok((self.read_i32()?, self.read_i32()?, self.read_i32()?))
    }

    fn read_string(&mut self) -> ParseResult<String> {
        let remaining = &self.data[self.pos..];

        let Some(length) = remaining.iter().position(|&b| b == 0) else {
            return self.error("String has no NUL terminator");
        };

        let bytes = self.read_bytes(length)?;
        self.read_u8()?; // Consume NUL.

        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    fn read_count(&mut self, name: &str) -> ParseResult<usize> {
        let offset = self.pos;
        let count = self.read_i32()?;

        if count < 0 || count as usize > MAX_ELEMENT_COUNT {
            return Err(SpriteParseError {
                offset,
                message: format!("Invalid {name}: {count}"),
            });
        }

        Ok(count as usize)
    }

    fn read_vertices(&mut self, count: usize) -> ParseResult<Vec<(i32, i32, i32)>> {
        let byte_count = count.checked_mul(12).ok_or_else(|| SpriteParseError {
            offset: self.pos,
            message: "Vertex data size overflow".into(),
        })?;

        self.require(byte_count)?;

        let mut vertices = Vec::with_capacity(count);

        for _ in 0..count {
            vertices.push(self.read_point3()?);
        }

        Ok(vertices)
    }

    fn read_triangles(&mut self, count: usize) -> ParseResult<Vec<MeshTriangle>> {
        let byte_count = count.checked_mul(16).ok_or_else(|| SpriteParseError {
            offset: self.pos,
            message: "Triangle data size overflow".into(),
        })?;

        self.require(byte_count)?;

        let mut triangles = Vec::with_capacity(count);

        for _ in 0..count {
            triangles.push(MeshTriangle {
                color: self.read_i32()?,
                indices: [self.read_i32()?, self.read_i32()?, self.read_i32()?],
            });
        }

        Ok(triangles)
    }
}

// -----------------------------------------------------------------------------
// Parse one sprite payload
// -----------------------------------------------------------------------------

/// Parse a TempleOS CSprite byte stream.
///
/// Elements are stored consecutively. Each begins with one type byte.
/// Bit 7 is the selection flag; bits 0-6 contain the element type.
/// The stream ends with a zero type byte.
pub fn parse_sprite(data: &[u8]) -> ParseResult<Sprite> {
    let mut reader = Reader::new(data);
    let mut elements = Vec::new();

    loop {
        let element_offset = reader.pos;
        let raw_type = reader.read_u8()?;

        let type_code = raw_type & TYPE_MASK;
        let selected = raw_type & 0x80 != 0;

        if type_code == SPT_END {
            return Ok(Sprite {
                elements,
                bytes_consumed: reader.pos,
                trailing_bytes: data.len() - reader.pos,
            });
        }

        if type_code > SPT_TEXT_DIAMOND {
            return Err(SpriteParseError {
                offset: element_offset,
                message: format!("Unknown sprite type: {type_code}"),
            });
        }

        let kind = match type_code {
            SPT_COLOR => SpriteElementData::Color {
                color: reader.read_u8()?,
            },

            SPT_DITHER_COLOR => SpriteElementData::DitherColor {
                color: reader.read_u16()?,
            },

            SPT_THICK => SpriteElementData::Thick {
                thickness: reader.read_i32()?,
            },

            SPT_PLANAR_SYMMETRY | SPT_LINE | SPT_RECT | SPT_ARROW => {
                let (x1, y1) = reader.read_point2()?;
                let (x2, y2) = reader.read_point2()?;

                match type_code {
                    SPT_PLANAR_SYMMETRY => SpriteElementData::PlanarSymmetry { x1, y1, x2, y2 },
                    SPT_LINE => SpriteElementData::Line { x1, y1, x2, y2 },
                    SPT_RECT => SpriteElementData::Rect { x1, y1, x2, y2 },
                    _ => SpriteElementData::Arrow { x1, y1, x2, y2 },
                }
            }

            SPT_TRANSFORM_ON => SpriteElementData::TransformOn,

            SPT_TRANSFORM_OFF => SpriteElementData::TransformOff,

            SPT_SHIFT | SPT_PT | SPT_FLOOD_FILL | SPT_FLOOD_FILL_NOT => {
                let (x, y) = reader.read_point2()?;

                match type_code {
                    SPT_SHIFT => SpriteElementData::Shift { x, y },
                    SPT_PT => SpriteElementData::Point { x, y },
                    SPT_FLOOD_FILL => SpriteElementData::FloodFill { x, y },
                    _ => SpriteElementData::FloodFillNot { x, y },
                }
            }

            SPT_POLYPT => {
                let count = reader.read_i32()?;

                if count < 0 || count as usize > MAX_ELEMENT_COUNT {
                    return Err(SpriteParseError {
                        offset: element_offset,
                        message: format!("Invalid poly-point count: {count}"),
                    });
                }

                let x = reader.read_i32()?;
                let y = reader.read_i32()?;

                let packed_size = (count as usize)
                    .checked_mul(3)
                    .and_then(|n| n.checked_add(7))
                    .map(|n| n / 8)
                    .ok_or_else(|| SpriteParseError {
                        offset: reader.pos,
                        message: "Poly-point data size overflow".into(),
                    })?;

                SpriteElementData::PolyPoint {
                    count,
                    x,
                    y,
                    packed_data: reader.read_bytes(packed_size)?,
                }
            }

            SPT_POLYLINE => {
                let count = reader.read_count("polyline point count")?;
                let mut points = Vec::with_capacity(count);

                for _ in 0..count {
                    points.push(reader.read_point2()?);
                }

                SpriteElementData::Polyline { points }
            }

            SPT_ROTATED_RECT => {
                let (x1, y1) = reader.read_point2()?;
                let (x2, y2) = reader.read_point2()?;
                let angle = reader.read_f64()?;

                SpriteElementData::RotatedRect {
                    x1,
                    y1,
                    x2,
                    y2,
                    angle,
                }
            }

            SPT_CIRCLE => {
                let (x, y) = reader.read_point2()?;
                let radius = reader.read_i32()?;

                SpriteElementData::Circle { x, y, radius }
            }

            SPT_ELLIPSE | SPT_POLYGON => {
                let (x, y) = reader.read_point2()?;
                let width = reader.read_i32()?;
                let height = reader.read_i32()?;
                let angle = reader.read_f64()?;

                if type_code == SPT_ELLIPSE {
                    SpriteElementData::Ellipse {
                        x,
                        y,
                        width,
                        height,
                        angle,
                    }
                } else {
                    let sides = reader.read_i32()?;

                    SpriteElementData::Polygon {
                        x,
                        y,
                        width,
                        height,
                        angle,
                        sides,
                    }
                }
            }

            SPT_BSPLINE2 | SPT_BSPLINE2_CLOSED | SPT_BSPLINE3 | SPT_BSPLINE3_CLOSED => {
                let count = reader.read_count("B-spline point count")?;
                let points = reader.read_vertices(count)?;

                SpriteElementData::BSpline {
                    count: count as i32,
                    points,
                    dimensions: if type_code == SPT_BSPLINE2 || type_code == SPT_BSPLINE2_CLOSED {
                        2
                    } else {
                        3
                    },
                    closed: type_code == SPT_BSPLINE2_CLOSED || type_code == SPT_BSPLINE3_CLOSED,
                }
            }

            SPT_BITMAP => {
                let (x, y) = reader.read_point2()?;
                let width = reader.read_i32()?;
                let height = reader.read_i32()?;

                if width < 0 || height < 0 {
                    return Err(SpriteParseError {
                        offset: element_offset,
                        message: format!("Invalid bitmap dimensions: {width}x{height}"),
                    });
                }

                let row_stride =
                    (width as usize)
                        .checked_add(7)
                        .ok_or_else(|| SpriteParseError {
                            offset: reader.pos,
                            message: "Bitmap row stride overflow".into(),
                        })?
                        & !7;

                let pixel_count =
                    row_stride
                        .checked_mul(height as usize)
                        .ok_or_else(|| SpriteParseError {
                            offset: reader.pos,
                            message: "Bitmap data size overflow".into(),
                        })?;

                SpriteElementData::Bitmap {
                    x,
                    y,
                    width,
                    height,
                    row_stride,
                    pixels: reader.read_bytes(pixel_count)?,
                }
            }

            SPT_MESH => {
                let vertex_count = reader.read_count("mesh vertex count")?;
                let triangle_count = reader.read_count("mesh triangle count")?;

                let vertices = reader.read_vertices(vertex_count)?;
                let triangles = reader.read_triangles(triangle_count)?;

                SpriteElementData::Mesh {
                    vertices,
                    triangles,
                }
            }

            SPT_SHIFTABLE_MESH => {
                let x = reader.read_i32()?;
                let y = reader.read_i32()?;
                let z = reader.read_i32()?;

                let vertex_count = reader.read_count("mesh vertex count")?;
                let triangle_count = reader.read_count("mesh triangle count")?;

                let vertices = reader.read_vertices(vertex_count)?;
                let triangles = reader.read_triangles(triangle_count)?;

                SpriteElementData::ShiftableMesh {
                    x,
                    y,
                    z,
                    vertices,
                    triangles,
                }
            }

            SPT_TEXT | SPT_TEXT_BOX | SPT_TEXT_DIAMOND => {
                let (x, y) = reader.read_point2()?;
                let text = reader.read_string()?;

                match type_code {
                    SPT_TEXT => SpriteElementData::Text { x, y, text },
                    SPT_TEXT_BOX => SpriteElementData::TextBox { x, y, text },
                    _ => SpriteElementData::TextDiamond { x, y, text },
                }
            }

            _ => unreachable!(),
        };

        elements.push(SpriteElement {
            type_code,
            selected,
            data: kind,
        });
    }
}

// -----------------------------------------------------------------------------
// Sprite type names
// -----------------------------------------------------------------------------

pub fn sprite_type_name(type_code: u8) -> &'static str {
    match type_code & TYPE_MASK {
        SPT_END => "END",
        SPT_COLOR => "COLOR",
        SPT_DITHER_COLOR => "DITHER_COLOR",
        SPT_THICK => "THICK",
        SPT_PLANAR_SYMMETRY => "PLANAR_SYMMETRY",
        SPT_TRANSFORM_ON => "TRANSFORM_ON",
        SPT_TRANSFORM_OFF => "TRANSFORM_OFF",
        SPT_SHIFT => "SHIFT",
        SPT_PT => "PT",
        SPT_POLYPT => "POLYPT",
        SPT_LINE => "LINE",
        SPT_POLYLINE => "POLYLINE",
        SPT_RECT => "RECT",
        SPT_ROTATED_RECT => "ROTATED_RECT",
        SPT_CIRCLE => "CIRCLE",
        SPT_ELLIPSE => "ELLIPSE",
        SPT_POLYGON => "POLYGON",
        SPT_BSPLINE2 => "BSPLINE2",
        SPT_BSPLINE2_CLOSED => "BSPLINE2_CLOSED",
        SPT_BSPLINE3 => "BSPLINE3",
        SPT_BSPLINE3_CLOSED => "BSPLINE3_CLOSED",
        SPT_FLOOD_FILL => "FLOOD_FILL",
        SPT_FLOOD_FILL_NOT => "FLOOD_FILL_NOT",
        SPT_BITMAP => "BITMAP",
        SPT_MESH => "MESH",
        SPT_SHIFTABLE_MESH => "SHIFTABLE_MESH",
        SPT_ARROW => "ARROW",
        SPT_TEXT => "TEXT",
        SPT_TEXT_BOX => "TEXT_BOX",
        SPT_TEXT_DIAMOND => "TEXT_DIAMOND",
        _ => "UNKNOWN",
    }
}

// -----------------------------------------------------------------------------
// Parse all resources in a Talons.HC file
// -----------------------------------------------------------------------------

/// Parse all DolDoc sprite resources from a Talons.HC byte buffer.
///
/// The input can be a Vec<u8>, &[u8], &Vec<u8>, or any other type
/// implementing AsRef<[u8]>.
///
/// Returns a map keyed by the resource BI number used in the source text.
/// For example, sprites.get(&4) retrieves BI=4.
///
/// This function does not read files or print anything.
pub fn parse_talons(data: impl AsRef<[u8]>) -> Result<HashMap<u32, Sprite>, TalonsParseError> {
    let data = data.as_ref();

    // The source text ends at its first NUL byte.
    let text_end = data
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| TalonsParseError {
            offset: 0,
            resource_id: None,
            message: "Missing source-text NUL terminator".into(),
        })?;

    let mut offset = text_end + 1;
    let mut sprites = HashMap::new();

    while offset < data.len() {
        let header_offset = offset;

        // DolDoc resource header:
        // U32 num, U32 flags, U32 size, U32 use_count.
        if data.len() - offset < 16 {
            return Err(TalonsParseError {
                offset,
                resource_id: None,
                message: "Incomplete DolDoc resource header".into(),
            });
        }

        let num = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());

        let size = u32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap()) as usize;

        let payload_start = offset + 16;

        let payload_end = payload_start
            .checked_add(size)
            .ok_or_else(|| TalonsParseError {
                offset: header_offset,
                resource_id: Some(num),
                message: "Resource payload size overflow".into(),
            })?;

        if payload_end > data.len() {
            return Err(TalonsParseError {
                offset: header_offset,
                resource_id: Some(num),
                message: format!("Resource payload exceeds file (size {size:#x})"),
            });
        }

        let payload = &data[payload_start..payload_end];

        let sprite = parse_sprite(payload).map_err(|error| TalonsParseError {
            offset: payload_start + error.offset,
            resource_id: Some(num),
            message: error.message,
        })?;

        if sprites.insert(num, sprite).is_some() {
            return Err(TalonsParseError {
                offset: header_offset,
                resource_id: Some(num),
                message: "Duplicate resource ID".into(),
            });
        }

        offset = payload_end;
    }

    Ok(sprites)
}
