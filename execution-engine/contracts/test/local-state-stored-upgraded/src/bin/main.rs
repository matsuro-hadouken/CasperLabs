#![no_std]
#![no_main]

#[no_mangle]
pub extern "C" fn call() {
    local_state_stored_upgraded::delegate();
}
