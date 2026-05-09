use pyo3::prelude::*;

#[pyfunction]
fn generate_content(content: &str, message: &str) -> String {
    let settings = makimono::Settings::default();

    makimono::generate_content(content, &settings, message.to_string(), &[]).unwrap()
}

#[pymodule]
fn pymakimono(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate_content, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::generate_content;

    #[test]
    fn generate_content_smoke_test() {
        let result = generate_content("### Latest Changes\n", "* New & first PR Feature");

        assert_eq!(result, "### Latest Changes\n\n* New & first PR Feature\n");
    }
}
