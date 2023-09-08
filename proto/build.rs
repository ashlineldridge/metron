fn main() -> Result<(), std::io::Error> {
    tonic_build::configure()
        .type_attribute(
            "LinearRatePlanSegment",
            "#[derive(serde::Deserialize, serde::Serialize)]",
        )
        .compile(&["src/metron.proto"], &["src"])?;

    Ok(())
}
