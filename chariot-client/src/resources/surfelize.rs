use crate::util::Pcg32Rng;
use crate::util::Rng;

pub fn sample_surfels(
    rng: &mut Pcg32Rng,
    verts: &[glam::Vec3],
    tex_coords: &[[f32; 2]],
    inds: &[u32],
    color_data: Option<&BaseColorData>,
    images: &[gltf::image::Data],
    sample_density: f64, // samples / area unit
) -> (Vec<glam::Vec3>, Vec<glam::Vec3>, Vec<glam::Vec3>) {
    let mut points = vec![];
    let mut normals = vec![];
    let mut colors = vec![];

    // TODO: hard-coded for now
    let cd = color_data.unwrap_or(&BaseColorData::Color([255, 255, 255, 255]));
    for tri in inds.chunks(3) {
        let (i, j, k) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let (pi, pj, pk) = (verts[i], verts[j], verts[k]);
        let (tci, tcj, tck) = (
            glam::Vec2::from_slice(&tex_coords[i]),
            glam::Vec2::from_slice(&tex_coords[j]),
            glam::Vec2::from_slice(&tex_coords[k]),
        );
        let (eij, eik) = (pj - pi, pk - pi);

        let n_ijk = eij.cross(eik).normalize();
        let a_ijk = (eij.cross(eik).length() * 0.5) as f64;

        let u: f32 = rng.next();
        let extra = if f64::fract(a_ijk * sample_density) > (u as f64) {
            1usize
        } else {
            0usize
        };
        let samples = (sample_density * a_ijk) as usize + extra;

        let (new_points, new_colors): (Vec<glam::Vec3>, Vec<glam::Vec3>) = (0..samples)
            .map(|_| {
                let (mut u0, mut u1): (f32, f32) = (rng.next(), rng.next());
                if u0 + u1 > 1.0 {
                    u0 = 1.0 - u0;
                    u1 = 1.0 - u1;
                }

                let u2 = 1.0 - u0 - u1;
                let tc = tci * u0 + tcj * u1 + tck * u2;
                let color = cd.sample(images, tc);

                (pi + eij * u0 + eik * u1, color)
            })
            .unzip();

        let new_normals = vec![n_ijk; samples];

        points.extend(new_points);
        normals.extend(new_normals);
        colors.extend(new_colors);
    }

    println!("\t\tGenerated {} surfel samples", points.len());
    (points, normals, colors)
}

fn sample_image(image: &gltf::image::Data, tc: glam::Vec2) -> [u8; 4] {
    let size = glam::uvec2(image.width, image.height);
    let sizef = size.as_vec2();
    let xyf = tc * sizef;
    let xy = xyf
        .as_uvec2()
        .clamp(glam::UVec2::ZERO, size - glam::UVec2::ONE);
    match image.format {
        gltf::image::Format::R8G8B8 => {
            let idx = ((xy[1] * size[0] + xy[0]) * 3) as usize;
            [
                image.pixels[idx],
                image.pixels[idx + 1],
                image.pixels[idx + 2],
                255,
            ]
        }
        gltf::image::Format::R8G8B8A8 => {
            let idx = ((xy[1] * size[0] + xy[0]) * 4) as usize;
            if idx >= image.pixels.len() {
                println!("{}, {}", xy[0], xy[1]);
                println!("{}, {}", size[0], size[1]);
            }

            [
                image.pixels[idx],
                image.pixels[idx + 1],
                image.pixels[idx + 2],
                255,
            ]
        }
        _ => panic!("TODO: trying to sample unimplemented format"),
    }
}

pub enum BaseColorData {
    ImageIndex(usize),
    Color([u8; 4]),
}

impl BaseColorData {
    fn sample(&self, images: &[gltf::image::Data], tc: glam::Vec2) -> glam::Vec3 {
        let pixel = match self {
            Self::ImageIndex(i) => {
                let img = &images[*i];
                sample_image(img, tc)
            }
            Self::Color(color) => *color,
        };

        glam::vec3(pixel[0] as f32, pixel[1] as f32, pixel[2] as f32) / 256.0
    }
}
