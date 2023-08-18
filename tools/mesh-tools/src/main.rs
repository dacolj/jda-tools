use byteorder::ReadBytesExt;
use clap::{Arg, Command};
use rand::Rng;
use std::collections::HashMap;
use std::fs::File;
use std::io::Cursor;
use std::io::{Read, Write};
use std::time::Instant;
extern crate nalgebra as na;
use na::{Point3, Vector3};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

struct Mesh {
    vertices: Vec<Point3<f64>>,
    triangles: Vec<[usize; 3]>,
    normals: Vec<Vector3<f64>>,
}

struct Cloud {
    vertices: Vec<Point3<f64>>,
    normals: Vec<Vector3<f64>>,
}

fn load_stl(filename: &str) -> Result<Mesh, std::io::Error> {
    let buffer = std::fs::read(filename)?;

    let mut cursor = Cursor::new(buffer);
    let mut header = [0 as u8; 80];
    cursor.read_exact(&mut header)?;

    let triangles_count = cursor.read_u32::<byteorder::LittleEndian>()? as usize;
    let mut triangles = vec![[0; 3]; triangles_count];
    let mut normals: Vec<Vector3<f64>> = vec![];
    normals.reserve_exact(triangles_count);
    let mut vertices: Vec<Point3<f64>> = vec![];
    let mut vertice_map: HashMap<u64, Vec<usize>> = HashMap::new();
    let mut last_idx = 0;

    for triangle in triangles.iter_mut() {
        let n = Vector3::<f64>::new(
            cursor.read_f32::<byteorder::LittleEndian>()? as f64,
            cursor.read_f32::<byteorder::LittleEndian>()? as f64,
            cursor.read_f32::<byteorder::LittleEndian>()? as f64,
        );
        normals.push(n);

        let v1 = Point3::<f64>::new(
            cursor.read_f32::<byteorder::LittleEndian>()? as f64,
            cursor.read_f32::<byteorder::LittleEndian>()? as f64,
            cursor.read_f32::<byteorder::LittleEndian>()? as f64,
        );
        let v2 = Point3::<f64>::new(
            cursor.read_f32::<byteorder::LittleEndian>()? as f64,
            cursor.read_f32::<byteorder::LittleEndian>()? as f64,
            cursor.read_f32::<byteorder::LittleEndian>()? as f64,
        );
        let v3 = Point3::<f64>::new(
            cursor.read_f32::<byteorder::LittleEndian>()? as f64,
            cursor.read_f32::<byteorder::LittleEndian>()? as f64,
            cursor.read_f32::<byteorder::LittleEndian>()? as f64,
        );
        let _padding = cursor.read_u16::<byteorder::LittleEndian>()?;

        let tab = [&v1, &v2, &v3];
        for i in 0..3 {
            let mut s = DefaultHasher::new();
            tab[i][0].to_be_bytes().hash(&mut s);
            tab[i][1].to_be_bytes().hash(&mut s);
            tab[i][2].to_be_bytes().hash(&mut s);
            let hash = s.finish();
            let mut found = false;
            match vertice_map.get_mut(&hash) {
                Some(v) => {
                    for idx in v.iter() {
                        if vertices[*idx] == *tab[i] {
                            triangle[i] = *idx;
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        vertices.push(*tab[i]);
                        v.push(last_idx);
                        triangle[i] = last_idx;
                        last_idx += 1;
                    }
                }
                None => {
                    vertices.push(*tab[i]);
                    vertice_map.insert(hash, vec![last_idx]);
                    triangle[i] = last_idx;
                    last_idx += 1;
                }
            }
        }
    }

    Ok(Mesh {
        vertices: vertices,
        triangles: triangles,
        normals: normals,
    })
}

fn compute_normal_per_vertex(mesh: &Mesh) -> Vec<Vector3<f64>> {
    // Compute normal by vertex
    #[derive(Clone)]
    struct Data {
        normal: Vector3<f64>,
        angle_rad: f64,
    }
    let mut data_vec: Vec<Vec<Data>> = vec![vec![]; mesh.vertices.len()];
    for triangle in mesh.triangles.iter() {
        // Compute angles and normal
        let a = &mesh.vertices[triangle[0]];
        let b = &mesh.vertices[triangle[1]];
        let c = &mesh.vertices[triangle[2]];
        let v1 = b - a;
        let v2 = c - b;
        let v3 = a - c;
        let angles = [v1.angle(&v2), v2.angle(&v3), v3.angle(&v1)];
        let normal = v1.cross(&v2);

        // Fill data_vec
        for i in 0..3 {
            data_vec[triangle[i]].push(Data {
                normal: normal.clone(),
                angle_rad: angles[i],
            });
        }
    }

    let mut normals: Vec<Vector3<f64>> = vec![];
    normals.reserve_exact(mesh.vertices.len());
    for data in data_vec.iter() {
        let mut normal = Vector3::<f64>::new(0.0, 0.0, 0.0);
        for d in data.iter() {
            normal += d.angle_rad * d.normal;
        }
        normal.normalize_mut();
        normals.push(normal);
    }

    normals
}

fn _shift_cloud_along_normals(
    vertices: &mut Vec<Point3<f64>>,
    normals: &Vec<Vector3<f64>>,
    shift: f64,
) -> () {
    for i in 0..vertices.len() {
        vertices[i] = vertices[i] + shift * normals[i];
    }
}

fn export_cloud_with_shift(
    filename: &str,
    cloud: &Cloud,
    shift: f64,
) -> Result<(), std::io::Error> {
    let mut file = File::create(filename)?;
    for i in 0..cloud.vertices.len() {
        let n = &cloud.normals[i];
        let pt = cloud.vertices[i] + shift * n;
        write!(file, "{} {} {} {} {} {}\n", pt.x, pt.y, pt.z, n.x, n.y, n.z).unwrap();
    }
    Ok(())
}

fn sample_point(mesh: &Mesh, sample_count: usize, verbose: bool) -> Result<Cloud, std::io::Error> {
    // Compute mesh surface
    let mut total_surface = 0.0;
    for idx in 0..mesh.triangles.len() {
        let triangle = &mesh.triangles[idx];
        let origin = &mesh.vertices[triangle[0]];
        let v1 = mesh.vertices[triangle[1]] - origin;
        let v2 = mesh.vertices[triangle[2]] - origin;
        total_surface += v1.cross(&v2).norm();
    }
    total_surface /= 2.0;

    let mut rng = rand::thread_rng();
    let density = (sample_count as f64) / total_surface;

    if verbose {
        println!("Surface {}", total_surface);
        println!("Density {}", density);
    }

    let mut vertices: Vec<Point3<f64>> = vec![];
    vertices.reserve_exact(sample_count);
    let mut normals: Vec<Vector3<f64>> = vec![];
    normals.reserve_exact(sample_count);

    let mut assigned = 0;
    for idx in 0..mesh.triangles.len() {
        let triangle = &mesh.triangles[idx];
        let origin = &mesh.vertices[triangle[0]];
        let v1 = mesh.vertices[triangle[1]] - origin;
        let v2 = mesh.vertices[triangle[2]] - origin;
        let surface = v1.cross(&v2).norm() / 2.0;

        let samples_assigned = {
            let frac = surface * density;
            let samples_assigned = frac as u32;
            if (frac - (samples_assigned as f64)) >= rng.gen::<f64>() {
                samples_assigned + 1
            } else {
                samples_assigned
            }
        };

        assigned += samples_assigned;


        let tri_normals = [
            &mesh.normals[triangle[0]],
            &mesh.normals[triangle[1]],
            &mesh.normals[triangle[2]],
        ];
        for _ in 0..samples_assigned {
            let mut s = rng.gen::<f64>();
            let mut t = rng.gen::<f64>();
            if s + t > 1.0 {
                s = 1.0 - s;
                t = 1.0 - t;
            }
            let pt = origin + s * v1 + t * v2;
            let normal: Vector3<f64> =
                ((1.0 - s - t) * tri_normals[0] + s * tri_normals[1] + t * tri_normals[2])
                    .normalize();
            vertices.push(pt);
            normals.push(normal);
        }
    }

    if verbose {
        println!(
            "Generate {} sample, initial requested {}",
            assigned, sample_count
        );
    }

    Ok(Cloud {
        vertices: vertices,
        normals: normals,
    })
}

fn main() {
    let matches = Command::new("mesh_tools")
        .arg(
            Arg::new("input")
                .help("Input file")
                .required(true)
                .short('i'),
        )
        .arg(
            Arg::new("output")
                .help("Output file")
                .required(true)
                .short('o'),
        )
        .arg(
            Arg::new("subsample")
                .help("subsample")
                .required(true)
                .value_parser(clap::value_parser!(usize))
                .short('s'),
        )
        .arg(
            Arg::new("offset")
                .help("offset")
                .required(false)
                .value_parser(clap::value_parser!(f64))
                .short('f'),
        )
        .arg(
            Arg::new("verbose")
                .help("versose")
                .required(false)
                .action(clap::ArgAction::SetTrue)
                .short('v'),
        )
        .get_matches();

    let input_filename = matches.get_one::<String>("input").unwrap();
    let output_filename = matches.get_one::<String>("output").unwrap();
    let samples_count = matches.get_one::<usize>("subsample").unwrap();
    let offset = matches.get_one::<f64>("offset");
    let verbose = matches.get_flag("verbose");

    let start: Instant = Instant::now();
    let mut mesh = load_stl(input_filename.as_str()).unwrap();
    if verbose {
        println!("Load time: {:?}", start.elapsed());
        println!("Polygone count: {}", mesh.triangles.len());
        println!("Vertex count: {}", mesh.vertices.len());
    }

    let start: Instant = Instant::now();
    mesh.normals = compute_normal_per_vertex(&mesh);
    if verbose {
        println!("Per vertex normal computation time: {:?}", start.elapsed());
    }

    let start: Instant = Instant::now();
    let cloud = sample_point(&mesh, *samples_count, verbose).unwrap();
    if verbose {
        println!("Sampling time: {:?}", start.elapsed());
    }

    let start: Instant = Instant::now();
    export_cloud_with_shift(output_filename.as_str(), &cloud, *offset.unwrap_or(&0.0)).unwrap();
    if verbose {
        println!("Save time: {:?}", start.elapsed());
    }
}
