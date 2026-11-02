use presence_rs::Patch;

#[derive(Patch)]
struct Account {
    #[patch(skip)]
    id: u64,
    name: String,
}

fn main() {
    let _ = AccountPatch {
        id: Some(1),
        name: None,
    };
}
