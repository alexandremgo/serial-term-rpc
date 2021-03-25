// TODO: read https://doc.rust-lang.org/book/ch14-01-release-profiles.html

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/serial_terminal.proto")?;
    Ok(())
}