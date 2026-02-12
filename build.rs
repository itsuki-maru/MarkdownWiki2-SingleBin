extern crate embed_resource;

fn main() {
    embed_resource::compile("res/app.rc", &["res"])
        .manifest_optional()
        .unwrap();
}
