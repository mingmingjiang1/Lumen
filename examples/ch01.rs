/*
    普通类型（可以作为 Value）, 也可以作为引用类型（可以作为 Reference），
    但是不能作为指针类型（不能作为 Pointer），因为它们的大小是固定的，编译器在编译时就知道了。
    比如像u8、u16、u32、u64、i8、i16、i32、i64、f32、f64等都是普通容器类型。
*/
use std::{alloc::Layout, ops::Deref, ptr};

struct Meters(f64);

/*
    Trait Deref 用于解引用操作符 `*`，它允许我们定义一个类型的解引用行为。通过实现 Deref trait，我们可以让自定义类型像指针一样使用解引用操作符，从而访问其内部数据。
    在这个例子中，我们为 Meters 类型实现了 Deref trait，使得我们可以通过解引用操作符 `*` 来访问 Meters 内部的 f64 数据。
*/

/*
    实现 Deref trait，使得 Meters 类型可以像 f64 类型一样使用方法和运算符。
*/
impl Deref for Meters {
    type Target = f64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/*
    特殊容器类型，内部包含一个裸指针和两个 usize 字段，用于表示一个动态大小的字符串。由于其大小在编译时无法确定，因此它不能作为 Value 或 Pointer 类型，只能作为 Reference 类型使用。
*/
#[derive(Debug)]
struct MyStr {
    ptr: *const u8,
    // cap: usize,
    len: usize,
}

impl MyStr {
    // fn new() -> Self {
    //     Self {
    //         ptr: std::ptr::null(),
    //         cap: 0,
    //         len: 0,
    //     }
    // }

    fn from(str: &str) -> Self {
        // Implementation for creating MyStr from a string
        let bytes = str.as_bytes();
        let len = bytes.len();
        let layout = Layout::array::<u8>(len).unwrap();
        let ptr = unsafe { std::alloc::alloc(layout) };
        if ptr.is_null() {
            panic!("Failed to allocate memory");
        }
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
        }
        Self { ptr, len }
    }

    // str == [u8], 也就是一个胖指针，包含了一个裸指针和一个长度字段
    fn as_str(&self) -> &str {
        // Implementation for converting MyStr to &str
        unsafe {
            let slice = std::slice::from_raw_parts(self.ptr, self.len);
            std::str::from_utf8_unchecked(slice)
        }
    }
}

impl Deref for MyStr {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        // Implementation for dereferencing MyStr to str
        self.as_str()
    }
}

/*
 *可以+裸指针，可以+胖指针，也可以+引用类型，前提是实现了Deref trait。
*/

fn take(s: &str) {
    println!("take: {}", s);
}


struct MyVec<T> {
    ptr: *const T,
    len: usize,
}

impl<T> MyVec<T> {
    // fn new() -> Self {
    //     Self {
    //         ptr: std::ptr::null(),
    //         len: 0,
    //     }
    // }

    // fn from(vec: Vec<T>) -> Self {
    //     let len = vec.len();
    //     let ptr = vec.as_ptr();
    //     std::mem::forget(vec); // Prevent Vec from being dropped
    //     Self { ptr, len }
    // }

    // fn as_slice(&self) -> &[T] {
    //     unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    // }
}

fn main() {
    // 这里是允许的，因为 Meters 是一个普通类型，它的大小是固定的，编译器在编译时就知道了。
    let m = Meters(5.0);
    let m_ref = &m; // 作为引用类型
    let m_ptr: *const Meters = &m; // 作为指针类型
    println!("Meters: {}", m.0);
    println!("Meters: {}", m_ref.0);
    println!("Meters: {}", unsafe { (*m_ptr).0 });
    /*
       为了能够使用 `sqrt` 方法和 `+` 运算符，我们需要为 `Meters` 类型实现相应的 trait。因为m本身并不支持这些操作，
       所以我们需要通过实现 `Deref` trait 来将 `Meters` 类型解引用为 `f64` 类型，从而能够使用 `f64` 的方法和运算符。
       有点类似继承的概念，但是Rust中没有继承，只有组合和trait实现。
       如果没有实现Deref，是无法解引用的
    */
    println!("{}", m.sqrt());
    println!("{}", *m + 1.0);

    println!("-------------------");

    let s: MyStr = MyStr::from("Hello, world!");
    let s_ref = &s; // 作为引用类型
    let s_ptr: *const MyStr = &s; // 作为指针类型
    println!("MyStr: {:?}", s_ref);
    println!("MyStr: {:?}", s_ptr);

    /*
         如果没有实现Deref，是无法解引用的 或者者说，无法使用 `*` 操作符来访问 MyStr 内部的数据。
         因为 str 是一个动态大小类型（DST），它的大小在编译时无法确定，所以不能直接作为值类型使用。
    */
    let t = (*s).len();
    // println!("MyStr: {:?}", *s); // error

    // 这里是允许的，因为 MyStr 实现了 Deref trait，可以解引用为 str 类型，从而可以作为引用类型使用。
    take(&s);
}
