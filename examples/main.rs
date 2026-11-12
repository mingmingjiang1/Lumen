use std::alloc::{Layout, alloc, dealloc};
use std::ops::Deref;
use std::ptr;

struct Str {
    ptr: *const u8,
    cap: usize,
    len: usize,
}

#[derive(Debug)]
struct User {
    id: u64,
    // age: u16,
    flag: bool,
}

fn test() {
    let layout = Layout::new::<User>();
    let ptr: *mut User = unsafe { alloc(layout) } as *mut User;
    unsafe {
        ptr::write(ptr, User { id: 1, flag: true });
    }

    unsafe {
        let user = &*ptr; // 从裸指针生成引用（你要保证对齐、有效、生命周期）
        println!("{:#?}", user);
    }
}

impl Str {
    pub fn new() -> Self {
        Self {
            ptr: ptr::null_mut(),
            cap: 0,
            len: 0,
        }
    }

    // 从&str构造，即接收只读引用
    pub fn from(s: &str) -> Self {
        let src = s.as_bytes();
        let len = src.len();
        if len == 0 {
            return Self::new();
        }
        // 分配内存
        let layout = Layout::from_size_align(len, 1).unwrap();
        let ptr: *mut u8 = unsafe { alloc(layout) };
        if ptr.is_null() {
            panic!("Failed to allocate memory");
        }
        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr(), ptr, len);
        }
        Self { ptr, cap: len, len }
    }

    pub fn as_str(&self) -> &str {
        unsafe {
            let slice = std::slice::from_raw_parts(self.ptr, self.len);
            str::from_utf8_unchecked(slice)
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Deref for Str {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

#[derive(Debug)]
enum MegStatus {
    Failed(String),
    // Success { val: String, status: u8 },
    // Other(u8, u8, u8),
}

fn test2(s: &str) {
    println!("Length: {}", s.len());
    println!("Content: {}", s);
    
}

fn main() {
    let s: Str = Str::from("Hello, world!");
    // 传参是Str的引用，通过deref解引用(所谓解引用：把指向Str（特殊结构体）的指针转换成胖指针（胖指针：堆字节起始地址 + 长度）)，deref的入参是原类型的瘦指针，返回值是胖指针
    test2(&s);
    let status = MegStatus::Failed(String::from("Error occurred"));
    match status {
        MegStatus::Failed(err) => println!("Error: {}", err),
        // MegStatus::Success { val, status } => println!("Success: {}, Status: {}", val, status),
        // MegStatus::Other(a, b, c) => println!("Other: {}, {}, {}", a, b, c),
    }
}
