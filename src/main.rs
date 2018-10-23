extern crate winapi;
extern crate user32;

use kernel32::GetModuleHandleW;
use winapi::HWND;
// use std::io::Write;
use std::process;
use std::ptr;

use user32::{
    DefWindowProcW,
    RegisterClassW,
    CreateWindowExW,
    TranslateMessage,
    DispatchMessageW,
    GetMessageW,
    MessageBoxW,
    SetParent,
};

use winapi::winuser::{
    MSG,
    MB_OK,
    WNDCLASSW,
    CS_OWNDC,
    CS_HREDRAW,
    CS_VREDRAW,
    CW_USEDEFAULT,
    WS_OVERLAPPEDWINDOW,
    WS_VISIBLE,
    WM_KEYUP
};


fn win32_str( v : &str ) -> Vec<u16>
{
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::iter::once;

    OsStr::new(v).encode_wide().chain(once(0)).collect()
}


fn show_message( msg : String )
{
    unsafe
    {
        MessageBoxW(ptr::null_mut(), win32_str(msg.as_str()).as_ptr(),
                    win32_str("Info Message").as_ptr(),
                    MB_OK);
    }
}

fn throw_fatal_error( msg : &str )
{
    unsafe {
        MessageBoxW(ptr::null_mut(), win32_str(msg).as_ptr(),
                    win32_str("Fatal Error!").as_ptr(),
                    MB_OK);
        // std::io::stdout().write(msg.as_bytes()) ;
        process::exit(1);
    }
}

struct WinRect
{
    pub x : i32, pub y : i32, pub width : i32, pub height : i32,
}

impl WinRect
{
    #[allow(dead_code)]
    fn new( x : i32, y : i32, width : i32, height : i32 ) -> Self
    {
        WinRect{ x : x, y : y, width : width, height : height }
    }

    fn some( x : i32, y : i32, width : i32, height : i32 ) -> Option<Self>
    {
        Some(WinRect{ x : x, y : y, width : width, height : height })
    }

}

struct Win32Window
{
    hwnd : HWND
}

pub unsafe extern "system" fn win_proc( hwnd : HWND, umsg : u32, wparam : u64, lparam : i64 ) -> i64
{

    match umsg {
        WM_KEYUP => {
            let scancode = (lparam & 0x00ff0000) >> 16; // extract scancode
            match scancode {
                0x01 => { // Escape went up
                    show_message("ESCAPE PRESSED!".to_owned());
                    return 0;
                },
                0x1c => { // Enter went up
                    show_message("ENTER PRESSED!".to_owned());
                    return 0;
                },
                _ => {
                    show_message(format!("KEY PRESSED: {:x?}", scancode));
                    return 0;
                }
            }
        }
        _ => {}
    }

    return DefWindowProcW( hwnd, umsg, wparam, lparam );
}

impl Win32Window
{
    fn create( name : &str, title : &str, placement : Option<WinRect> )
               -> Result<Win32Window, String>
    {
        unsafe
        {
            let window_name = win32_str(name);
            let window_title = win32_str(title);
            let hinstance =  GetModuleHandleW(ptr::null_mut());
            let wnd_class = WNDCLASSW {
                style : CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc : Some( win_proc  ),
                hInstance : hinstance,
                lpszClassName : window_name.as_ptr(),
                cbClsExtra : 0,
                cbWndExtra : 0,
                hIcon : ptr::null_mut(),
                hCursor : ptr::null_mut(),
                hbrBackground : ptr::null_mut(),
                lpszMenuName : ptr::null_mut()
            };

            RegisterClassW(&wnd_class);

            let ( _wx, _wy, _ww, _wh ) = match placement {
                Some(p) => ( p.x, p.y, p.width, p.height ),
                None => ( CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT )
            };

            let hwnd = CreateWindowExW(
                0, window_name.as_ptr(), window_title.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                _wx, _wy, // position
                _ww, _wh,
                ptr::null_mut(), //  parent 
                ptr::null_mut(),
                hinstance,
                ptr::null_mut()
            );

            if hwnd.is_null() {
                return Err(String::from("Could not create Win32 window!"));
            }else{
                return Ok(Win32Window{ hwnd : hwnd });
            }
        }
    }

    #[allow(dead_code)]
    fn set_parent( &self, other : &Win32Window )
    {
        unsafe { SetParent(self.hwnd, other.hwnd)} ;
    }

    fn handle_messages(&self) -> bool
    {
        unsafe
        {
            let mut msg : MSG = std::mem::uninitialized();
            let r = GetMessageW(&mut msg as *mut MSG, self.hwnd, 0, 0 );
            if r > 0 {
                TranslateMessage( &msg as *const MSG );
                DispatchMessageW( &msg as *const MSG );

                return true;
            }else if r == -1 {
                return false;
            }else{
                return true;
            }
        }
    }
}

fn main ()
{
    // use std::time::Duration;
    // use std::thread::sleep;
    use std::thread;
    let t1 = thread::spawn( move || {
        let window1 = Win32Window::create("HelloApp", "Hello world!", WinRect::some(0,0,200,100))
            .map_err(|e| throw_fatal_error(e.as_str())).ok().unwrap();

        loop {
            if ! window1.handle_messages() {
                break;
            }
        }
    });

    t1.join().ok().unwrap(); 
}
