// delegate_impl
//
// Copyright (C) 2019 SOFe
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![feature(proc_macro_hygiene)]
#![allow(macro_expanded_macro_exports_accessed_by_absolute_paths)]

use static_assertions::assert_impl_all as assert_impl;

mod module {
    use delegate_impl::delegate;
    #[delegate(path = "module")]
    pub trait Foo {
        fn len(&self) -> usize;
    }
}

#[derive(Default)]
struct Bar {
    qux: String,
}

crate::delegate_foo!(Bar : qux ; );

// assert_impl!(Bar, Foo);

#[test]
fn test_equal() {
    use module::Foo;
    let bar = Bar { qux: "abc".into() };
    assert_eq!(bar.len(), 3);
}
