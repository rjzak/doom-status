use vergen_gitcl::{BuildBuilder, Emitter, GitclBuilder};

fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/1.ico");
        res.compile().unwrap();
    }

    let mut git = GitclBuilder::default();
    let git = git.all().describe(false, true, None).build().unwrap();

    Emitter::default()
        .add_instructions(&BuildBuilder::all_build().unwrap())
        .unwrap()
        .add_instructions(&git)
        .unwrap()
        .emit()
        .unwrap();
}
