use rustfft::{num_complex::Complex, num_traits::Zero, FFTplanner, FFT};

use std::sync::Arc;

fn freq(i: usize, sample_rate: usize, bin_size: usize) -> f32 {
	i as f32 * sample_rate as f32 / bin_size as f32
}

pub struct FrequencyAnalyser {
    sample_rate: usize,
    bin_size: usize,
    ffft: Arc<dyn FFT<f32>>,
	ifft: Arc<dyn FFT<f32>>,
	values: Option<Vec<Complex<f32>>>
}

impl FrequencyAnalyser {
    pub fn new(sample_rate: usize, bin_size: usize) -> Self {
		// println!("{}: {}", sample_rate, bin_size);
        let ffft = FFTplanner::new(false).plan_fft(bin_size);
        let ifft = FFTplanner::new(true).plan_fft(bin_size);
        Self {
            sample_rate,
            bin_size,
            ffft,
			ifft,
			values: None
        }
	}
	
	pub fn analise(&mut self, v: &[f32]) {
		assert!(v.len() == self.bin_size, "Data to analise shouldnt be different from the bin size");
		let mut v: Vec<Complex<f32>> = v.iter().copied().map(|x|Complex::new(x, 0.)).collect();
		let mut out = vec![Complex::zero(); self.bin_size];
		self.ffft.process(&mut v, &mut out);
		self.values = Some(out);
	}

    pub fn freq<'a>(&'a mut self) -> Option<Frequencies<'a>> {
		if let Some(x) = &mut self.values {
			Some(Frequencies::new(x.as_mut_slice(), self.sample_rate, self.bin_size))
		}else{
			None
		}
	}

	pub fn result(&mut self) -> Option<Vec<f32>> {
		if let Some(x) = &mut self.values {
			let mut out = vec![Complex::zero(); self.bin_size];
			self.ifft.process(x.as_mut_slice(), &mut out);
			Some(out.into_iter().map(|x| x.re / self.bin_size as f32).collect())
		}else{
			None
		}
	}
}

#[derive(Clone, Copy)]
pub enum ApplyKind {
	Less,
	LessEq,
	Eq(f32),
	GreaterEq,
	Greater
}

pub struct Frequencies<'a> {
    f: &'a mut [Complex<f32>],
    sample_rate: usize,
    bin_size: usize,
}

impl<'a> Frequencies<'a> {
    fn new(f: &'a mut [Complex<f32>], sample_rate: usize, bin_size: usize) -> Self {
        Self {
            f,
            sample_rate,
            bin_size,
        }
	}
	
	/// Applies a function to all frequencies that match
	/// The function takes for input the curreent frequency and its amplitude, and returns the new value
	pub fn apply<F: Fn(f32, f32) -> f32>(&mut self, kind: ApplyKind, frequency: f32, f: F) {
		for i in 0..self.f.len() {
			let curr_freq = freq(i, self.sample_rate, self.bin_size);
			let op = match kind {
				ApplyKind::Less => curr_freq < frequency,
				ApplyKind::LessEq => curr_freq <= frequency,
				ApplyKind::Eq(epsilon) => (curr_freq - frequency).abs() < epsilon,
				ApplyKind::GreaterEq => curr_freq >= frequency,
				ApplyKind::Greater => curr_freq > frequency,
			};
			if op {
				let v = self.f[i];
				self.f[i] = Complex::new(f(curr_freq, v.re), f(curr_freq, v.im))
			}
		}
	}
}
