use std::io::{Read, Seek, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use hemtt_common::io::{ReadExt, WriteExt};
use indexmap::IndexMap;

use crate::{Error, Face, Point};

#[derive(Debug)]
pub struct LOD {
    pub version_major: u32,
    pub version_minor: u32,
    pub resolution: f32,
    pub points: Vec<Point>,
    pub face_normals: Vec<(f32, f32, f32)>,
    pub faces: Vec<Face>,
    pub taggs: IndexMap<String, Box<[u8]>>,
}

impl LOD {
    /// Reads a LOD from a given input stream.
    ///
    /// # Errors
    /// [`std::io::Error`] if an IO error occurs.
    pub fn read<I: Read + Seek>(input: &mut I) -> Result<Self, Error> {
        let mut buffer = [0; 4];
        input.read_exact(&mut buffer)?;
        if &buffer != b"P3DM" {
            return Err(Error::UnsupportedLODType(
                String::from_utf8_lossy(&buffer).to_string(),
            ));
        }

        let version_major = input.read_u32::<LittleEndian>()?;
        let version_minor = input.read_u32::<LittleEndian>()?;

        let num_points = input.read_u32::<LittleEndian>()?;
        let num_face_normals = input.read_u32::<LittleEndian>()?;
        let num_faces = input.read_u32::<LittleEndian>()?;

        input.bytes().nth(3);

        let mut points: Vec<Point> = Vec::with_capacity(num_points as usize);
        let mut face_normals: Vec<(f32, f32, f32)> = Vec::with_capacity(num_face_normals as usize);
        let mut faces: Vec<Face> = Vec::with_capacity(num_faces as usize);

        for _i in 0..num_points {
            points.push(Point::read(input)?);
        }

        for _i in 0..num_face_normals {
            face_normals.push((
                input.read_f32::<LittleEndian>()?,
                input.read_f32::<LittleEndian>()?,
                input.read_f32::<LittleEndian>()?,
            ));
        }

        for _i in 0..num_faces {
            faces.push(Face::read(input)?);
        }

        input.read_exact(&mut buffer)?;
        if &buffer != b"TAGG" {
            return Err(Error::UnexpectedBytesTagg(
                String::from_utf8_lossy(&buffer).to_string(),
            ));
        }

        let mut taggs: IndexMap<String, Box<[u8]>> = IndexMap::new();

        loop {
            input.bytes().next();

            let name = input.read_cstring()?;
            let size = input.read_u32::<LittleEndian>()?;
            let mut buffer = vec![0; size as usize].into_boxed_slice();
            input.read_exact(&mut buffer)?;

            if name == "#EndOfFile#" {
                break;
            }

            taggs.insert(name, buffer);
        }

        let resolution = input.read_f32::<LittleEndian>()?;

        Ok(Self {
            version_major,
            version_minor,
            resolution,
            points,
            face_normals,
            faces,
            taggs,
        })
    }

    /// Writes the LOD to a given output stream.
    ///
    /// # Errors
    /// [`std::io::Error`] if an IO error occurs.
    pub fn write<O: Write>(&self, output: &mut O) -> Result<(), Error> {
        let points_count = u32::try_from(self.points.len())
            .map_err(|_| Error::ExceededMaxPointCount(self.points.len() as u64))?;
        let face_normals_count = u32::try_from(self.face_normals.len())
            .map_err(|_| Error::ExceededMaxFaceNormalCount(self.face_normals.len() as u64))?;
        let faces_count = u32::try_from(self.faces.len())
            .map_err(|_| Error::ExceededMaxFaceCount(self.faces.len() as u64))?;

        output.write_all(b"P3DM")?;
        output.write_u32::<LittleEndian>(self.version_major)?;
        output.write_u32::<LittleEndian>(self.version_minor)?;
        output.write_u32::<LittleEndian>(points_count)?;
        output.write_u32::<LittleEndian>(face_normals_count)?;
        output.write_u32::<LittleEndian>(faces_count)?;
        output.write_all(b"\0\0\0\0")?;

        for point in &self.points {
            point.write(output)?;
        }

        for normal in &self.face_normals {
            output.write_f32::<LittleEndian>(normal.0)?;
            output.write_f32::<LittleEndian>(normal.1)?;
            output.write_f32::<LittleEndian>(normal.2)?;
        }

        for face in &self.faces {
            face.write(output)?;
        }

        output.write_all(b"TAGG")?;

        for (name, buffer) in &self.taggs {
            let buffer_len = u32::try_from(buffer.len())
                .map_err(|_| Error::ExceededTaggLength(buffer.len() as u64))?;
            output.write_all(&[1])?;
            output.write_cstring(name)?;
            output.write_u32::<LittleEndian>(buffer_len)?;
            output.write_all(buffer)?;
        }

        output.write_cstring("\x01#EndOfFile#")?;
        output.write_u32::<LittleEndian>(0)?;

        output.write_f32::<LittleEndian>(self.resolution)?;

        Ok(())
    }
}
