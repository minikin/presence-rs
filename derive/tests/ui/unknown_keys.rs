use presence_rs::Patch;

#[derive(Patch)]
#[patch(rename = AccountUpdate)]
struct Account {
    #[patch(nullable)]
    owner: String,
    #[patch(skip, flatten)]
    note: Option<String>,
}

fn main() {}
