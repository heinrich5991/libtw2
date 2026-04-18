use clap::App;
use libtw2_packer::UnexpectedEnd;
use libtw2_packer::Unpacker;
use libtw2_tools::warn_stderr::Stderr;
use std::io;
use std::io::Read as _;
use std::io::Write as _;

fn main() -> Result<(), io::Error> {
    let _ = App::new("Teeworlds variable-length integer decoding")
        .about(
            "Decodes stdin as a list of Teeworlds variable-length integers\
                to big-endian 32-bit integers",
        )
        .get_matches();

    let mut stdin = Vec::new();
    io::stdin().read_to_end(&mut stdin)?;
    let mut unpacker = Unpacker::new(&stdin);
    let mut result = Vec::new();
    while !unpacker.is_empty() {
        result.extend_from_slice(
            &unpacker
                .read_int(&mut Stderr)
                .map_err(|UnexpectedEnd| io::Error::other("unexpected end"))?
                .to_be_bytes(),
        );
    }
    io::stdout().write_all(&result)?;
    Ok(())
}
