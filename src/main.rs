#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle] // この関数の名前修飾をしない
pub extern "C" fn _start() -> ! {
    // リンカはデフォルトで `_start` という名前の関数を探すので、
    // この関数がエントリポイントとなる
    loop {}
}

/// パニック時に呼び出される関数
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
