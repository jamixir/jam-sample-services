// Links the service ELF for the JAM instruction set and writes it as a GP v0.7.2 service code
// blob: E(↕metadata, Y-program), with refine at pc 0 and accumulate at pc 5 (Appendix B).
use polkavm_common::program::{ProgramBlob, ProgramParts};
use polkavm_linker::{Config, TargetInstructionSet};

const PAGE: u32 = 4096;

// GP general natural encoding
fn natural(x: u64) -> Vec<u8> {
    for l in 0..8u32 {
        if x < 1u64 << (7 * (l + 1)) {
            let high = (x >> (8 * l)) as u8;
            let mut out = vec![(256u32 - (1u32 << (8 - l))) as u8 + high];
            out.extend_from_slice(&x.to_le_bytes()[..l as usize]);
            return out;
        }
    }
    let mut out = vec![255];
    out.extend_from_slice(&x.to_le_bytes());
    out
}

fn fixed(x: u64, n: usize) -> Vec<u8> {
    x.to_le_bytes()[..n].to_vec()
}

fn round_up(x: u32, to: u32) -> u32 {
    x.div_ceil(to) * to
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args[1] == "target" {
        let path = polkavm_linker::target_json_path(Default::default()).expect("target json");
        println!("{}", path.display());
        return;
    }
    let (elf_path, out_path) = (&args[1], &args[2]);
    let elf = std::fs::read(elf_path).expect("read elf");

    let mut config = Config::default();
    config.set_strip(true);
    config.set_dispatch_table(vec![b"refine".to_vec(), b"accumulate".to_vec()]);
    let polkavm = polkavm_linker::program_from_elf(config, TargetInstructionSet::JamV1, &elf)
        .expect("link");

    let parts = ProgramParts::from_bytes(polkavm.into()).expect("parts");
    let blob = ProgramBlob::from_parts(parts.clone()).expect("blob");

    // deblob: E(|j|) ⌢ E1(z) ⌢ E(|c|) ⌢ Ez(j) ⌢ c ⌢ k
    let jump_table: Vec<u32> = blob.jump_table().iter().map(|pc| pc.0).collect();
    let max = jump_table.iter().copied().max().unwrap_or(0);
    let z = (1..=4).find(|n| (max as u64) < 1u64 << (8 * n)).unwrap();
    let mut code = natural(jump_table.len() as u64);
    code.push(z as u8);
    code.extend(natural(blob.code().len() as u64));
    for entry in &jump_table {
        code.extend(fixed(*entry as u64, z));
    }
    code.extend_from_slice(blob.code());
    code.extend_from_slice(blob.bitmask());

    // Y: E3(|o|) ⌢ E3(|w|) ⌢ E2(z) ⌢ E3(s) ⌢ o ⌢ w ⌢ E4(|c|) ⌢ c
    let mut o = parts.ro_data.to_vec();
    o.resize(parts.ro_data_size as usize, 0);
    let w = parts.rw_data.to_vec();
    let zero_pages = (round_up(parts.rw_data_size, PAGE) - round_up(w.len() as u32, PAGE)) / PAGE;

    let mut program = fixed(o.len() as u64, 3);
    program.extend(fixed(w.len() as u64, 3));
    program.extend(fixed(zero_pages as u64, 2));
    program.extend(fixed(parts.stack_size as u64, 3));
    program.extend(&o);
    program.extend(&w);
    program.extend(fixed(code.len() as u64, 4));
    program.extend(code);

    // Formula (9.4) v0.7.2: E(↕m, c), with an empty metadata
    let mut service = natural(0);
    service.extend(program);
    std::fs::write(out_path, &service).expect("write");

    eprintln!(
        "{} bytes: ro {} rw {} zero pages {} stack {} code {} jump table {:?}",
        service.len(),
        o.len(),
        w.len(),
        zero_pages,
        parts.stack_size,
        blob.code().len(),
        jump_table
    );
}
