use std::{collections::HashMap, io::Cursor};

use image::{io::Reader, DynamicImage};

#[allow(dead_code)]
#[repr(usize)]
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum ColorComponent {
    Red = 0,
    Green = 1,
    Blue = 2,
    Alpha = 3,
}

#[derive(Clone)]
pub struct Image {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl Image {
    pub fn new_with_data(data: Vec<u8>) -> Self {
        let image = Self::decode_data(data);

        Self {
            data: image.to_rgba8().into_vec(),
            width: image.width(),
            height: image.height(),
        }
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn get_data_ref(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn get_histogram(&self) -> HashMap<ColorComponent, [u32; 256]> {
        let mut histogram = HashMap::new();

        for component in &[
            ColorComponent::Red,
            ColorComponent::Green,
            ColorComponent::Blue,
        ] {
            histogram.insert(*component, [0; 256]);
        }

        for pixel in self.data.chunks(4) {
            histogram.get_mut(&ColorComponent::Red).unwrap()[pixel[0] as usize] += 1;
            histogram.get_mut(&ColorComponent::Green).unwrap()[pixel[1] as usize] += 1;
            histogram.get_mut(&ColorComponent::Blue).unwrap()[pixel[2] as usize] += 1;
        }

        histogram
    }

    pub fn get_grayscale_histogram(&self) -> [u32; 256] {
        let mut histogram = [0; 256];

        for pixel in self.data.chunks(4) {
            let grayscale = (pixel[0] as f32 * 0.2126
                + pixel[1] as f32 * 0.7152
                + pixel[2] as f32 * 0.0722) as u8;

            histogram[grayscale as usize] += 1;
        }

        histogram
    }

    pub fn get_equalized_image(&self) -> Self {
        let histogram = self.get_histogram();
        let mut cdf = HashMap::new();
        let mut min = HashMap::new();

        for component in &[
            ColorComponent::Red,
            ColorComponent::Green,
            ColorComponent::Blue,
        ] {
            cdf.insert(*component, [0; 256]);
            min.insert(*component, u32::MAX);
        }

        for component in &[
            ColorComponent::Red,
            ColorComponent::Green,
            ColorComponent::Blue,
        ] {
            let mut sum = 0;

            for i in 0..256 {
                sum += histogram.get(component).unwrap()[i];
                cdf.get_mut(component).unwrap()[i] = sum;
                if sum < min[component] {
                    min.insert(*component, sum);
                }
            }
        }

        let pixels = self.width * self.height;

        let mut data = self.data.clone();
        for (i, chunk) in self.data.chunks(4).enumerate() {
            for component in &[
                ColorComponent::Red,
                ColorComponent::Green,
                ColorComponent::Blue,
            ] {
                let up =
                    (cdf[component][chunk[*component as usize] as usize] - min[component]) as f32;
                let down = (pixels - min[component]) as f32;
                let val = (up / down * 255.0).round() as u8;
                data[(i * 4) + *component as usize] = val;
            }
        }

        Self {
            data,
            width: self.width,
            height: self.height,
        }
    }

    pub fn get_stretched_image(&self) -> Self {
        let histogram = self.get_histogram();
        let mut min = HashMap::new();
        let mut max = HashMap::new();

        for component in &[
            ColorComponent::Red,
            ColorComponent::Green,
            ColorComponent::Blue,
        ] {
            min.insert(*component, 255u8);
            max.insert(*component, 0u8);
        }

        for component in &[
            ColorComponent::Red,
            ColorComponent::Green,
            ColorComponent::Blue,
        ] {
            for i in 0..256 {
                if histogram.get(component).unwrap()[i] > 0 {
                    if (i as u8) < *min.get(component).unwrap() {
                        min.insert(*component, i as u8);
                    }

                    if (i as u8) > *max.get(component).unwrap() {
                        max.insert(*component, i as u8);
                    }
                }
            }
        }

        let mut data = self.data.clone();
        for (i, chunk) in self.data.chunks(4).enumerate() {
            for component in &[
                ColorComponent::Red,
                ColorComponent::Green,
                ColorComponent::Blue,
            ] {
                let min = min[component];
                let max = max[component];
                data[(i * 4) + *component as usize] =
                    ((chunk[*component as usize] - min) as f32 / (max - min) as f32 * 255.0) as u8;
            }
        }

        Self {
            data,
            width: self.width,
            height: self.height,
        }
    }

    pub fn threshold(&self, (low, high): (u8, u8)) -> Self {
        let mut data = self.data.clone();
        for (i, chunk) in self.data.chunks(4).enumerate() {
            let mut val = 0;
            for component in &[
                ColorComponent::Red,
                ColorComponent::Green,
                ColorComponent::Blue,
            ] {
                if chunk[*component as usize] >= low && chunk[*component as usize] <= high {
                    val = 255;
                }
            }

            for component in &[
                ColorComponent::Red,
                ColorComponent::Green,
                ColorComponent::Blue,
            ] {
                data[(i * 4) + *component as usize] = val;
            }
        }

        Self {
            data,
            width: self.width,
            height: self.height,
        }
    }

    pub fn percent_black_selection(&self, percent: f32) -> Self {
        let histogram = self.get_grayscale_histogram();

        let pixels = ((self.width * self.height) as f32 * percent).floor() as u32;
        let mut sum = 0;
        let mut threshold = 0;
        for i in 0..256 {
            sum += histogram[i];
            if sum >= pixels {
                threshold = i;
                break;
            }
        }

        self.threshold((threshold as u8, 255))
    }

    pub fn mean_iterative_selection(&self) -> Self {
        let histogram = self.get_grayscale_histogram();
        let mut mean = 0.0;
        let mut prev_mean = 0.0;
        let mut count = 0;

        for i in 0..256 {
            mean += i as f32 * histogram[i] as f32;
            count += histogram[i];
        }

        mean /= count as f32;

        while (mean - prev_mean).abs() > 0.01 {
            let mut low_mean = 0.0;
            let mut low_count = 0;
            let mut high_mean = 0.0;
            let mut high_count = 0;

            for i in 0..256 {
                if (i as f32) < mean {
                    low_mean += i as f32 * histogram[i] as f32;
                    low_count += histogram[i];
                } else {
                    high_mean += i as f32 * histogram[i] as f32;
                    high_count += histogram[i];
                }
            }

            low_mean /= low_count as f32;
            high_mean /= high_count as f32;

            prev_mean = mean;
            mean = (low_mean + high_mean) / 2.0;
        }

        self.threshold((mean as u8, 255))
    }

    pub fn entropy_selection(&self) -> Self {
        let histogram = self
            .get_grayscale_histogram()
            .map(|x| x as f32 / (self.width * self.height) as f32);
        let mut max_sum = std::f32::MIN;
        let mut f;
        let mut pt = 0.0;

        let mut max_low = histogram[0];
        let mut max_high;
        let mut ht = 0.0;
        let mut ht_total = 0.0;

        for i in 0..256 {
            if histogram[i] > 0.0 {
                ht_total -= histogram[i] * (histogram[i] as f32).log2();
            }
        }

        let mut threshold = 0;
        for i in 0..256 {
            pt += histogram[i];
            max_low = max_low.max(histogram[i]);
            max_high = if i < 255 {
                histogram[i + 1]
            } else {
                histogram[i]
            };

            for j in i + 2..256 {
                if histogram[j] > max_high {
                    max_high = histogram[j];
                }
            }

            if histogram[i] > 0.0 {
                ht -= histogram[i] * (histogram[i] as f32).log2();
            }

            f = ht * (pt as f32).log2() / (ht_total * (max_low as f32).log2())
                + (1.0 - ht / ht_total) * (1.0 - pt as f32).log2() / (max_high as f32).log2();

            if f > max_sum as f32 {
                max_sum = f;
                threshold = i;
            }
        }

        self.threshold((threshold as u8, 255))
    }

    pub fn minimum_error_selection(&self) -> Self {
        let histogram = self
            .get_grayscale_histogram()
            .map(|x| x as f32 / (self.width * self.height) as f32);

        let mut min_value = std::f32::MAX;
        let mut j;
        let mut p1 = 0.0;
        let mut p2 = 0.0;
        let mut s1;
        let mut s2;
        let mut fv;
        let mut u1;
        let mut u2;
        let mut pi1 = 0.0;
        let mut pi2 = 0.0;

        for i in 0..256 {
            p2 += histogram[i];
            pi2 += i as f32 * histogram[i];
        }

        let mut threshold = 0;
        for i in 0..256 {
            p1 += histogram[i];
            p2 -= histogram[i];
            pi1 += i as f32 * histogram[i];
            pi2 -= i as f32 * histogram[i];

            u1 = if p1 > 0.0 {
                pi1 as f32 / p1 as f32
            } else {
                0.0
            };
            u2 = if p2 > 0.0 {
                pi2 as f32 / p2 as f32
            } else {
                0.0
            };

            s1 = 0.0;
            if p1 > 0.0 {
                for j in 0..=i {
                    fv = j as f32 - u1;
                    s1 += fv * fv * histogram[j] as f32;
                }
                s1 /= p1 as f32;
            }

            s2 = 0.0;
            if p2 > 0.0 {
                for j in i + 1..256 {
                    fv = j as f32 - u2;
                    s2 += fv * fv * histogram[j] as f32;
                }
                s2 /= p2 as f32;
            }

            j = 1.0 + 2.0 * ((p1 * s1.log2() - p1.log2()) + p2 * (s2.log2() - p2.log2()));
            if j == std::f32::NAN || j == -std::f32::INFINITY {
                continue;
            }

            if j < min_value {
                min_value = j;
                threshold = i;
            }
        }

        self.threshold((threshold as u8, 255))
    }

    pub fn fuzzy_minimum_error_selection(&self) -> Self {
        let histogram = self.get_grayscale_histogram();
        let mut min_error = std::f32::MAX;
        let mut threshold = 0;

        let mut max = 0;
        let mut min = 255;

        for i in 0..256 {
            if histogram[i] > 0 {
                if i > max {
                    max = i;
                }
                if i < min {
                    min = i;
                }
            }
        }

        let c = max - min;

        for t in 0..255 {
            let mut mu0 = 0.0;
            let mut c0 = 0;
            for i in 0..=t {
                mu0 += i as f32 * histogram[i] as f32;
                c0 += histogram[i];
            }
            mu0 /= c0 as f32;

            let mut mu1 = 0.0;
            let mut c1 = 0;
            for i in t + 1..256 {
                mu1 += i as f32 * histogram[i] as f32;
                c1 += histogram[i];
            }
            mu1 /= c1 as f32;

            let mut e = 0.0;
            for i in 0..=t {
                e += Self::shannon(c as f32 / (c as f32 + (i as f32 - mu0).abs()))
                    * histogram[i] as f32;
            }

            for i in t + 1..256 {
                e += Self::shannon(c as f32 / (c as f32 + (i as f32 - mu1).abs()))
                    * histogram[i] as f32;
            }

            e /= (self.width * self.height) as f32;

            if e < min_error {
                min_error = e;
                threshold = t;
            }
        }

        self.threshold((threshold as u8, 255))
    }
    fn shannon(x: f32) -> f32 {
        if x == 0.0 {
            0.0
        } else {
            -x * x.log2() - (1.0 - x) * (1.0 - x).log2()
        }
    }

    fn decode_data(data: Vec<u8>) -> DynamicImage {
        let reader = Reader::new(Cursor::new(&data[..]))
            .with_guessed_format()
            .expect("Couldn't guess file format.");

        reader.decode().expect("Unable to decode image.")
    }
}
