#[cfg(target_os = "windows")]
use anyhow::{Result, anyhow};
use std::sync::mpsc::{self, Receiver, Sender};
use std::collections::HashMap;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        UI::{
            Shell::*,
            WindowsAndMessaging::*,
        },
        Graphics::Gdi::*,
        System::LibraryLoader::*,
    },
};

use super::{Tray, TrayEvent, MenuItem};

/// Windows 平台托盘实现
pub struct WindowsTray {
    hwnd: HWND,
    event_receiver: Receiver<TrayEvent>,
    _event_sender: Sender<TrayEvent>,
    menu_items: HashMap<u32, String>,
    next_menu_id: u32,
}

const WM_TRAYICON: u32 = WM_USER + 1;
const TRAY_ID: u32 = 1;

impl WindowsTray {
    /// 创建隐藏窗口用于接收托盘消息
    fn create_hidden_window(event_sender: Sender<TrayEvent>) -> Result<HWND> {
        unsafe {
            let instance = GetModuleHandleW(None)?;
            
            let window_class = w!("SeeUTrayWindow");
            
            let wc = WNDCLASSW {
                lpfnWndProc: Some(Self::window_proc),
                hInstance: instance.into(),
                lpszClassName: window_class,
                ..Default::default()
            };
            
            let atom = RegisterClassW(&wc);
            if atom == 0 {
                return Err(anyhow!("Failed to register window class"));
            }
            
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                window_class,
                w!("SeeU Tray"),
                WINDOW_STYLE::default(),
                0, 0, 0, 0,
                None,
                None,
                instance,
                Some(&event_sender as *const _ as *const _),
            );
            
            if hwnd.0 == 0 {
                return Err(anyhow!("Failed to create window"));
            }
            
            Ok(hwnd)
        }
    }
    
    /// 窗口过程函数
    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_CREATE => {
                let create_struct = lparam.0 as *const CREATESTRUCTW;
                let event_sender = (*create_struct).lpCreateParams as *const Sender<TrayEvent>;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, event_sender as isize);
                LRESULT(0)
            }
            WM_TRAYICON => {
                let event_sender_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Sender<TrayEvent>;
                if !event_sender_ptr.is_null() {
                    let event_sender = &*event_sender_ptr;
                    
                    match lparam.0 as u32 {
                        WM_LBUTTONUP => {
                            let _ = event_sender.send(TrayEvent::LeftClick);
                        }
                        WM_RBUTTONUP => {
                            let _ = event_sender.send(TrayEvent::RightClick);
                        }
                        WM_LBUTTONDBLCLK => {
                            let _ = event_sender.send(TrayEvent::DoubleClick);
                        }
                        _ => {}
                    }
                }
                LRESULT(0)
            }
            WM_COMMAND => {
                let menu_id = (wparam.0 & 0xFFFF) as u32;
                let event_sender_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Sender<TrayEvent>;
                if !event_sender_ptr.is_null() {
                    let event_sender = &*event_sender_ptr;
                    let _ = event_sender.send(TrayEvent::MenuItemClick(menu_id.to_string()));
                }
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
    
    /// 创建托盘图标
    fn create_tray_icon(&self, icon: &[u8]) -> Result<()> {
        unsafe {
            // 这里应该从字节数据创建图标，简化实现使用默认图标
            let hicon = LoadIconW(None, IDI_APPLICATION)?;
            
            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: TRAY_ID,
                uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
                uCallbackMessage: WM_TRAYICON,
                hIcon: hicon,
                szTip: [0; 128],
                ..Default::default()
            };
            
            // 设置默认提示文本
            let tip = "SeeU Desktop";
            let tip_wide: Vec<u16> = tip.encode_utf16().chain(std::iter::once(0)).collect();
            let copy_len = std::cmp::min(tip_wide.len(), nid.szTip.len());
            nid.szTip[..copy_len].copy_from_slice(&tip_wide[..copy_len]);
            
            let result = Shell_NotifyIconW(NIM_ADD, &nid);
            if !result.as_bool() {
                return Err(anyhow!("Failed to add tray icon"));
            }
            
            Ok(())
        }
    }
}

impl Tray for WindowsTray {
    fn new(icon: &[u8], menu: Vec<MenuItem>) -> Result<Self> {
        let (event_sender, event_receiver) = mpsc::channel();
        let hwnd = Self::create_hidden_window(event_sender.clone())?;
        
        let mut tray = WindowsTray {
            hwnd,
            event_receiver,
            _event_sender: event_sender,
            menu_items: HashMap::new(),
            next_menu_id: 1000,
        };
        
        tray.create_tray_icon(icon)?;
        tray.update_menu(menu)?;
        
        Ok(tray)
    }
    
    fn set_tooltip(&mut self, text: &str) -> Result<()> {
        unsafe {
            let mut nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: TRAY_ID,
                uFlags: NIF_TIP,
                szTip: [0; 128],
                ..Default::default()
            };
            
            let tip_wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let copy_len = std::cmp::min(tip_wide.len(), nid.szTip.len());
            nid.szTip[..copy_len].copy_from_slice(&tip_wide[..copy_len]);
            
            let result = Shell_NotifyIconW(NIM_MODIFY, &nid);
            if !result.as_bool() {
                return Err(anyhow!("Failed to set tooltip"));
            }
            
            Ok(())
        }
    }
    
    fn update_menu(&mut self, menu: Vec<MenuItem>) -> Result<()> {
        // 简化实现：存储菜单项映射
        self.menu_items.clear();
        for item in menu {
            if !item.separator {
                self.menu_items.insert(self.next_menu_id, item.id);
                self.next_menu_id += 1;
            }
        }
        Ok(())
    }
    
    fn set_icon(&mut self, _icon: &[u8]) -> Result<()> {
        // TODO: 实现图标更新
        Ok(())
    }
    
    fn try_recv_event(&mut self) -> Option<TrayEvent> {
        self.event_receiver.try_recv().ok()
    }
    
    fn shutdown(self) -> Result<()> {
        unsafe {
            let nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: self.hwnd,
                uID: TRAY_ID,
                ..Default::default()
            };
            
            Shell_NotifyIconW(NIM_DELETE, &nid);
            DestroyWindow(self.hwnd);
        }
        Ok(())
    }
}
