use core::fmt;
use volatile::Volatile;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
// u8は8ビット（1バイト）なので、背景色と前景色をそれぞれ4ビットで表現することができます。これにより、1バイトで2つの色情報を格納できる。
struct ColorCode(u8);

// ColorCode::new(Color::Red, Color::Black)は、
// 背景が黒（0b0000_0000）、前景が赤（0b0000_0001）として組み合わせた
// u8値（0b0001、つまり1）を持つColorCodeを生成します。
impl ColorCode {
    // (background as u8) << 4
    // は、背景色（4ビット）の値を8ビットの変数にキャストしてから、左に4ビットシフト
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    // 単一のバイト（u8）を受け取り、それを適切な位置に書き込む処理
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                // 現在の行がいっぱいかをチェックし、いっぱいなら改行する
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                // volatileでコンパイラが書き込みを最適化して取り除いてしまわないことを保証
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                // 現在の列の位置を1つ進める
                self.column_position += 1;
            }
        }
    }

    // 文字列（&str）を受け取り、各バイトを処理するためにwrite_byteメソッドを呼び出し
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // 改行文字は改行する
                // 0x20..=0x7eはASCIIの印刷可能な文字（スペースからチルダまで）
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // それ以外の文字はスペースに置き換える
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        // すべての文字を一行上に持っていき（一番上の行は消去されます）
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        // 前の行の最初から始めるようにカーソルをリセット
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    // すべての文字を空白文字で書き換えることによって行をクリア
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub fn print_something() {
    use core::fmt::Write;
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_byte(b'H');
    writer.write_string("ello ");
    write!(writer, "The numbers are {} and {}", 42, 1.0 / 3.0).unwrap();
}
