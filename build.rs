use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(&["src/onebot_idl/onebot_frame.proto"], &["src/onebot_idl"])?;
    Ok(())
}