use crate::files;

pub fn args(args: &[String]) {
    let profiles = files::get_profiles();
    print!("{profiles}");
}
