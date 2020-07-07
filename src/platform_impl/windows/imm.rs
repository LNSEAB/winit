use crate::dpi::PhysicalPosition;
use winapi::{
    shared::{
        minwindef::{BOOL, DWORD, LPARAM, LPVOID},
        windef::HWND,
        windef::{POINT, RECT},
    },
    um::{
        imm::{ImmGetContext, ImmReleaseContext, CFS_CANDIDATEPOS, CFS_EXCLUDE, HIMC},
        winnt::LONG,
    },
};

pub const GCS_COMPSTR: DWORD = 0x0008;
pub const GCS_CURSORPOS: DWORD = 0x0080;
pub const GCS_RESULTSTR: DWORD = 0x0800;
pub const ISC_SHOWUICOMPOSITIONWINDOW: LPARAM = 0x80000000;

#[repr(C)]
#[allow(non_snake_case)]
struct CANDIDATEFORM {
    dwIndex: DWORD,
    dwStyle: DWORD,
    ptCurrentPos: POINT,
    rcArea: RECT,
}

extern "system" {
    fn ImmSetCandidateWindow(himc: HIMC, form: *mut CANDIDATEFORM) -> BOOL;
    fn ImmGetCompositionStringW(himc: HIMC, index: DWORD, lpbuf: LPVOID, buflen: DWORD) -> LONG;
}

pub struct Imc {
    hwnd: HWND,
    himc: HIMC,
}

impl Imc {
    pub fn get_context(hwnd: HWND) -> Self {
        let himc = unsafe { ImmGetContext(hwnd) };
        Self { hwnd, himc }
    }

    pub fn set_candidate_window_position(&self, position: PhysicalPosition<i32>) {
        unsafe {
            let pt = POINT {
                x: position.x,
                y: position.y,
            };
            let mut form = CANDIDATEFORM {
                dwStyle: CFS_CANDIDATEPOS,
                dwIndex: 0,
                ptCurrentPos: pt,
                rcArea: RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                },
            };
            ImmSetCandidateWindow(self.himc, &mut form);
            let mut form = CANDIDATEFORM {
                dwStyle: CFS_EXCLUDE,
                dwIndex: 0,
                ptCurrentPos: pt,
                rcArea: RECT {
                    left: pt.x,
                    top: pt.y,
                    right: pt.x,
                    bottom: pt.y,
                },
            };
            ImmSetCandidateWindow(self.himc, &mut form);
        }
    }

    pub fn get_composition_string(&self, index: DWORD) -> (String, usize) {
        unsafe {
            let byte_len = ImmGetCompositionStringW(self.himc, index, std::ptr::null_mut(), 0);
            let len = byte_len as usize / std::mem::size_of::<u16>();
            let mut buf = Vec::with_capacity(len);
            buf.set_len(len);
            ImmGetCompositionStringW(
                self.himc,
                index,
                buf.as_mut_ptr() as *mut _,
                byte_len as DWORD,
            );
            let s = String::from_utf16_lossy(&buf);
            let byte_pos =
                ImmGetCompositionStringW(self.himc, GCS_CURSORPOS, std::ptr::null_mut(), 0)
                    as usize
                    & 0xffff;
            let char_pos = byte_pos / std::mem::size_of::<u16>();
            let mut pos = 0;
            for (i, c) in s.char_indices() {
                pos += c.len_utf8();
                if i == char_pos {
                    break;
                }
            }
            (s, pos)
        }
    }
}

impl Drop for Imc {
    fn drop(&mut self) {
        unsafe {
            ImmReleaseContext(self.hwnd, self.himc);
        }
    }
}
