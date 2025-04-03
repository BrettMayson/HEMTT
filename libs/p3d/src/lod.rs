use std::io::{Read, Seek, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use hemtt_common::io::{ReadExt, WriteExt};
use serde::Serialize;

use crate::{Error, Face, Point};

#[derive(Debug, PartialEq, Serialize)]
pub struct LOD {
    pub version_major: u32,
    pub version_minor: u32,
    pub unknown_flags: u32,
    pub resolution: f32,
    pub type_name: String,
    pub points: Vec<Point>,
    pub face_normals: Vec<(f32, f32, f32)>,
    pub faces: Vec<Face>,
    pub taggs: Vec<(String, Box<[u8]>)>,
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

        let unknown_flags = input.read_u32::<LittleEndian>()?;

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

        let mut taggs: Vec<(String, Box<[u8]>)> = Vec::new();

        loop {
            input.bytes().next();

            let name = input.read_cstring()?;
            let size = input.read_u32::<LittleEndian>()?;
            let mut buffer = vec![0; size as usize].into_boxed_slice();
            input.read_exact(&mut buffer)?;

            if name == "#EndOfFile#" {
                break;
            }

            taggs.push((name, buffer));
        }

        let resolution = input.read_f32::<LittleEndian>()?;
        let type_name = Self::get_lod_type_from_resolution(resolution);

        Ok(Self {
            version_major,
            version_minor,
            unknown_flags,
            resolution,
            type_name,
            points,
            face_normals,
            faces,
            taggs,
        })
    }

    fn get_lod_type_from_resolution(resolution: f32) -> String {
        if (20000.0..30000.0).contains(&resolution) {
            return "Edit ".to_owned() + &(resolution - 20000.0).floor().to_string();
        }

        let type_name = Self::get_lod_type_from_resolution_match(resolution).to_string();
        if type_name != "Unknown" {
            return type_name;
        }

        if resolution > 1000.0 {
            return "Unknown ".to_owned() + &resolution.to_string();
        }

        "Resolution ".to_owned() + &resolution.to_string()
    }

    #[expect(
        clippy::float_cmp,
        reason = "All of the numbers should be safe for exact comparison"
    )]
    fn get_lod_type_from_resolution_match(resolution: f32) -> &'static str {
        // View positions
        if resolution == 1000.0 {
            return "View Gunner";
        }
        if resolution == 1100.0 {
            return "View Pilot";
        }
        if resolution == 1200.0 {
            return "View Cargo";
        }
        // Shadow volumes
        if resolution == 10000.0 {
            return "Shadow Volume 0";
        }
        if resolution == 10010.0 {
            return "Shadow Volume 10";
        }
        if resolution == 11000.0 {
            return "Shadow Buffer 0";
        }
        if resolution == 11010.0 {
            return "Shadow Buffer 10";
        }
        // Geometry types
        if resolution == 1e13 {
            return "Geometry";
        }
        if resolution == 2e13 {
            return "Geometry Buoyancy";
        }
        if resolution == 4e13 {
            return "Geometry PysX";
        }
        // Memory and special geometries
        if resolution == 1e15 {
            return "Memory";
        }
        if resolution == 2e15 {
            return "Land Contact";
        }
        if resolution == 3e15 {
            return "Roadway";
        }
        if resolution == 4e15 {
            return "Paths";
        }
        if resolution == 5e15 {
            return "Hit-points";
        }
        if resolution == 6e15 {
            return "View Geometry";
        }
        if resolution == 7e15 {
            return "Fire Geometry";
        }
        if resolution == 8e15 {
            return "View Cargo Geom.";
        }
        if resolution == 9e15 {
            return "View Cargo Fire Geom.";
        }
        // Commander, pilot, gunner views
        if resolution == 1e16 {
            return "View Commander";
        }
        if resolution == 1.1e16 {
            return "View Commander Geom.";
        }
        if resolution == 1.2e16 {
            return "View Commander Fire Geom.";
        }
        if resolution == 1.3e16 {
            return "View Pilot Geom.";
        }
        if resolution == 1.4e16 {
            return "View Pilot Fire Geom.";
        }
        if resolution == 1.5e16 {
            return "View Gunner Geom.";
        }
        if resolution == 1.6e16 {
            return "View Gunner Fire Geom.";
        }
        // Additional types
        if resolution == 1.7e16 {
            return "Sub Parts";
        }
        if resolution == 1.8e16 {
            return "Shadow Volume - View Cargo";
        }
        if resolution == 1.9e16 {
            return "Shadow Volume - View Pilot";
        }
        if resolution == 2e16 {
            return "Shadow Volume - View Gunner";
        }
        if resolution == 2.1e16 {
            return "Wreck";
        }

        "Unknown"
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
        output.write_u32::<LittleEndian>(self.unknown_flags)?;

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
