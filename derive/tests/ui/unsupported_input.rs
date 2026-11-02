use presence_rs::Patch;

#[derive(Patch)]
struct Pair(u8, u8);

#[derive(Patch)]
struct Unit;

#[derive(Patch)]
enum Choice {
    A,
}

#[derive(Patch)]
union Bits {
    whole: u32,
}

fn main() {}
