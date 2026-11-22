use nom::bytes::complete::{take, take_till};
use nom::error::{Error as NomError, ErrorKind};
use nom::multi::{count, many0};
use nom::number::complete::{le_f64, le_i32, le_u8, le_u16, le_u32};
use nom::{Err as NomErr, IResult, Parser};
use std::collections::HashMap;
use std::fmt;

type Bytes<'a> = &'a [u8];
type Res<'a, T> = IResult<Bytes<'a>, T>;

const TYPE_MASK: u8 = 0x7F;
const MAX_ELEMENT_COUNT: usize = 1_000_000;
const RESOURCE_HEADER_TAIL: usize = 4;

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

#[derive(Debug)]
pub struct Sprite {
    pub elements: Vec<SpriteElementData>,
}

#[derive(Debug)]
pub enum SpriteElementData {
    Color {
        color: u8,
    },
    Shift {
        x: i32,
        y: i32,
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
    Ignored,
}

#[derive(Debug)]
pub struct MeshTriangle {
    pub color: i32,
    pub indices: [i32; 3],
}

#[derive(Debug)]
pub struct TalonsParseError {
    pub offset: usize,
    pub resource_id: Option<u32>,
    pub message: String,
}

impl fmt::Display for TalonsParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.resource_id {
            Some(id) => write!(
                f,
                "Talons.HC parse error at {:#x} (BI={id}): {}",
                self.offset, self.message
            ),
            None => write!(
                f,
                "Talons.HC parse error at {:#x}: {}",
                self.offset, self.message
            ),
        }
    }
}

impl std::error::Error for TalonsParseError {}

fn failure<T>(at: Bytes<'_>, kind: ErrorKind) -> Res<'_, T> {
    Err(NomErr::Failure(NomError::new(at, kind)))
}

fn byte(i: Bytes<'_>) -> Res<'_, u8> {
    le_u8(i)
}

fn word(i: Bytes<'_>) -> Res<'_, u16> {
    le_u16(i)
}

fn long(i: Bytes<'_>) -> Res<'_, u32> {
    le_u32(i)
}

fn int(i: Bytes<'_>) -> Res<'_, i32> {
    le_i32(i)
}

fn float(i: Bytes<'_>) -> Res<'_, f64> {
    le_f64(i)
}

fn take_n(i: Bytes<'_>, n: usize) -> Res<'_, Bytes<'_>> {
    take(n).parse(i)
}

fn until_nul(i: Bytes<'_>) -> Res<'_, Bytes<'_>> {
    take_till(|b: u8| b == 0).parse(i)
}

fn point2(i: Bytes<'_>) -> Res<'_, (i32, i32)> {
    (int, int).parse(i)
}

fn point3(i: Bytes<'_>) -> Res<'_, (i32, i32, i32)> {
    (int, int, int).parse(i)
}

fn length(i: Bytes<'_>) -> Res<'_, usize> {
    let (rest, value) = int(i)?;
    if value < 0 || value as usize > MAX_ELEMENT_COUNT {
        return failure(i, ErrorKind::TooLarge);
    }
    Ok((rest, value as usize))
}

fn skip_string(i: Bytes<'_>) -> Res<'_, ()> {
    let (i, _) = until_nul(i)?;
    let (i, _) = byte(i)?;
    Ok((i, ()))
}

fn ignored(i: Bytes<'_>) -> Res<'_, SpriteElementData> {
    Ok((i, SpriteElementData::Ignored))
}

fn triangle(i: Bytes<'_>) -> Res<'_, MeshTriangle> {
    let (i, (color, a, b, c)) = (int, int, int, int).parse(i)?;
    Ok((
        i,
        MeshTriangle {
            color,
            indices: [a, b, c],
        },
    ))
}

type Geometry = (Vec<(i32, i32, i32)>, Vec<MeshTriangle>);

fn geometry(i: Bytes<'_>) -> Res<'_, Geometry> {
    let (i, vertex_count) = length(i)?;
    let (i, triangle_count) = length(i)?;
    let (i, vertices) = count(point3, vertex_count).parse(i)?;
    let (i, triangles) = count(triangle, triangle_count).parse(i)?;
    Ok((i, (vertices, triangles)))
}

fn bitmap(i: Bytes<'_>) -> Res<'_, SpriteElementData> {
    let (i, (x, y)) = point2(i)?;
    let (i, (width, height)) = (int, int).parse(i)?;
    if width < 0 || height < 0 {
        return failure(i, ErrorKind::Verify);
    }

    let row_stride = (width as usize + 7) & !7;
    let (i, pixels) = take_n(i, row_stride.saturating_mul(height as usize))?;

    Ok((
        i,
        SpriteElementData::Bitmap {
            x,
            y,
            width,
            height,
            row_stride,
            pixels: pixels.to_vec(),
        },
    ))
}

fn element_body(code: u8, i: Bytes<'_>) -> Res<'_, SpriteElementData> {
    match code {
        SPT_COLOR => {
            let (i, color) = byte(i)?;
            Ok((i, SpriteElementData::Color { color }))
        }
        SPT_DITHER_COLOR => {
            let (i, _) = word(i)?;
            ignored(i)
        }
        SPT_THICK => {
            let (i, _) = int(i)?;
            ignored(i)
        }
        SPT_LINE => {
            let (i, (x1, y1, x2, y2)) = (int, int, int, int).parse(i)?;
            Ok((i, SpriteElementData::Line { x1, y1, x2, y2 }))
        }
        SPT_PLANAR_SYMMETRY | SPT_RECT | SPT_ARROW => {
            let (i, _) = (int, int, int, int).parse(i)?;
            ignored(i)
        }
        SPT_TRANSFORM_ON | SPT_TRANSFORM_OFF => ignored(i),
        SPT_SHIFT => {
            let (i, (x, y)) = point2(i)?;
            Ok((i, SpriteElementData::Shift { x, y }))
        }
        SPT_PT | SPT_FLOOD_FILL | SPT_FLOOD_FILL_NOT => {
            let (i, _) = point2(i)?;
            ignored(i)
        }
        SPT_POLYPT => {
            let (i, points) = length(i)?;
            let (i, _) = point2(i)?;
            let (i, _) = take_n(i, (points * 3).div_ceil(8))?;
            ignored(i)
        }
        SPT_POLYLINE => {
            let (i, points) = length(i)?;
            let (i, points) = count(point2, points).parse(i)?;
            Ok((i, SpriteElementData::Polyline { points }))
        }
        SPT_ROTATED_RECT | SPT_ELLIPSE => {
            let (i, _) = ((int, int, int, int), float).parse(i)?;
            ignored(i)
        }
        SPT_CIRCLE => {
            let (i, _) = (int, int, int).parse(i)?;
            ignored(i)
        }
        SPT_POLYGON => {
            let (i, _) = ((int, int, int, int), float, int).parse(i)?;
            ignored(i)
        }
        SPT_BSPLINE2 | SPT_BSPLINE2_CLOSED | SPT_BSPLINE3 | SPT_BSPLINE3_CLOSED => {
            let (i, points) = length(i)?;
            let (i, _) = count(point3, points).parse(i)?;
            ignored(i)
        }
        SPT_BITMAP => bitmap(i),
        SPT_MESH => {
            let (i, (vertices, triangles)) = geometry(i)?;
            Ok((i, SpriteElementData::Mesh { vertices, triangles }))
        }
        SPT_SHIFTABLE_MESH => {
            let (i, (x, y, z)) = point3(i)?;
            let (i, (vertices, triangles)) = geometry(i)?;
            Ok((
                i,
                SpriteElementData::ShiftableMesh {
                    x,
                    y,
                    z,
                    vertices,
                    triangles,
                },
            ))
        }
        SPT_TEXT | SPT_TEXT_BOX | SPT_TEXT_DIAMOND => {
            let (i, _) = point2(i)?;
            let (i, _) = skip_string(i)?;
            ignored(i)
        }
        _ => failure(i, ErrorKind::Tag),
    }
}

fn element(i: Bytes<'_>) -> Res<'_, SpriteElementData> {
    let (after_tag, raw) = byte(i)?;
    let code = raw & TYPE_MASK;

    if code == SPT_END {
        return Err(NomErr::Error(NomError::new(i, ErrorKind::Eof)));
    }
    if code > SPT_TEXT_DIAMOND {
        return failure(i, ErrorKind::Tag);
    }

    element_body(code, after_tag)
}

fn end_marker(i: Bytes<'_>) -> Res<'_, ()> {
    let (rest, raw) = byte(i)?;
    if raw & TYPE_MASK != SPT_END {
        return failure(i, ErrorKind::Verify);
    }
    Ok((rest, ()))
}

fn sprite(i: Bytes<'_>) -> Res<'_, Sprite> {
    let (i, elements) = many0(element).parse(i)?;
    let (i, _) = end_marker(i)?;
    Ok((i, Sprite { elements }))
}

fn resource(i: Bytes<'_>) -> Res<'_, (u32, Bytes<'_>)> {
    let (i, (id, _flags, size)) = (long, long, long).parse(i)?;
    let (i, _) = take_n(i, RESOURCE_HEADER_TAIL)?;
    let (i, payload) = take_n(i, size as usize)?;
    Ok((i, (id, payload)))
}

fn offset_in(data: &[u8], part: &[u8]) -> usize {
    part.as_ptr() as usize - data.as_ptr() as usize
}

fn describe(
    data: &[u8],
    error: NomErr<NomError<Bytes<'_>>>,
    resource_id: Option<u32>,
) -> TalonsParseError {
    match error {
        NomErr::Error(inner) | NomErr::Failure(inner) => TalonsParseError {
            offset: offset_in(data, inner.input),
            resource_id,
            message: inner.code.description().to_string(),
        },
        NomErr::Incomplete(_) => TalonsParseError {
            offset: data.len(),
            resource_id,
            message: "Unexpected end of data".to_string(),
        },
    }
}

pub fn parse_talons(data: impl AsRef<[u8]>) -> Result<HashMap<u32, Sprite>, TalonsParseError> {
    let data = data.as_ref();

    let text_end = data
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| TalonsParseError {
            offset: 0,
            resource_id: None,
            message: "Missing source-text NUL terminator".to_string(),
        })?;

    let mut rest = &data[text_end + 1..];
    let mut sprites = HashMap::new();

    while !rest.is_empty() {
        let (after, (id, payload)) = resource(rest).map_err(|error| describe(data, error, None))?;
        let (_, parsed) = sprite(payload).map_err(|error| describe(data, error, Some(id)))?;

        if sprites.insert(id, parsed).is_some() {
            return Err(TalonsParseError {
                offset: offset_in(data, rest),
                resource_id: Some(id),
                message: "Duplicate resource ID".to_string(),
            });
        }

        rest = after;
    }

    Ok(sprites)
}
