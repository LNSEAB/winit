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

pub const IMM_ERROR_NODATA: LONG = -1;
pub const IMM_ERROR_GENERAL: LONG = -2;
pub const GCS_COMPSTR: DWORD = 0x0008;
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

    pub fn get_composition_string(&self, index: DWORD) -> Option<String> {
        unsafe {
            let len = ImmGetCompositionStringW(self.himc, index, std::ptr::null_mut(), 0);
            if len == IMM_ERROR_NODATA || len == IMM_ERROR_GENERAL {
                return None;
            }
            let len = len as usize;
            let mut buf = Vec::with_capacity(len / 2);
            buf.set_len(len / 2);
            let ret = ImmGetCompositionStringW(
                self.himc,
                index,
                buf.as_mut_ptr() as *mut _,
                len as DWORD,
            );
            if ret == IMM_ERROR_GENERAL {
                return None;
            }
            Some(String::from_utf16_lossy(&buf))
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
