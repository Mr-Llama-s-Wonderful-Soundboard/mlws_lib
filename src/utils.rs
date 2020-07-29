pub fn bytes_as_str(b: usize) -> String {
	if b < 1024 {
		format!("{} B", b)
	}else if b < 1024 * 1024 {
		format!("{:.2} kB", b as f64 / 1024.)
	}else if b < 1024 * 1024 * 1024 {
		format!("{:.2} MB", b as f64 / (1024. * 1024.))
	}else {
		format!("{:.2} GB", b as f64 / (1024. * 1024. * 1024.))
	}
}