use ic_cdk::caller;

use crate::get_token;

#[inline(always)]
pub fn controller_guard() -> Result<(), String> {
    if get_token().controllers.contains(&caller()) {
        Ok(())
    } else {
        Err(String::from("The caller is not a controller"))
    }
}
