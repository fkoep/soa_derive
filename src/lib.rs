#![recursion_limit="1024"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

/// TODO 
///
/// * Implement more Slice/Vec-like methods.
/// * When to use #[inline]?
/// * Work with generic structs?
#[proc_macro_derive(Soa)]
pub fn soa(input: TokenStream) -> TokenStream {
    let s = input.to_string();
    let ast = syn::parse_derive_input(&s).unwrap();
    let gen = impl_soa(&ast);
    gen.parse().unwrap()
}

fn impl_soa(ast: &syn::DeriveInput) -> quote::Tokens {
    let vis = ast.vis.clone();
    let ident = ast.ident.clone();
    let soa_ident = quote::Ident::from(format!("{}Soa", ident));
    let soa_ref_ident = quote::Ident::from(format!("{}SoaRef", ident));
    let soa_mut_ident = quote::Ident::from(format!("{}SoaMut", ident));
    let soa_iter_ident = quote::Ident::from(format!("{}SoaIter", ident));
    let soa_iter_mut_ident = quote::Ident::from(format!("{}SoaIterMut", ident));

    let fields = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) 
            if !fields.is_empty() 
        => {
            // TODO if vis is visible, assert that all fields are visible!
            fields.clone()
        }
        _ => panic!("DeriveSoa only works on non-generic, non-tuple, non-unit structs!"),
    };
    let mut field_idents = Vec::new();
    let mut field_tys = Vec::new();
    for field in &fields {
        field_idents.push(field.ident.as_ref().unwrap().clone());
        field_tys.push(field.ty.clone());
    }

    let (field_idents, field_tys) = (&field_idents, &field_tys);
    let field_idents2 = field_idents; // FIXME shouldn't be needed?

    quote!{
        // ++++++++++++++++++++ SoaRef ++++++++++++++++++++
        
        #[allow(unused)]
        #[derive(Clone)]
        #vis struct #soa_ref_ident<'a> {
            #(#field_idents: &'a #field_tys),*
        }

        impl<'a> #soa_ref_ident<'a> {
            // TODO Naming? to_owned?
            #vis fn to_value(&self) -> #ident {
                #soa_ident::new_value(#(self.#field_idents.clone()),*)
            }
        }

        // ++++++++++++++++++++ SoaMut ++++++++++++++++++++
        
        #[allow(unused)]
        #vis struct #soa_mut_ident<'a> {
            #(#field_idents: &'a mut #field_tys),*
        }

        impl<'a> #soa_mut_ident<'a> {
            // TODO Naming? to_owned?
            #vis fn to_value(&self) -> #ident {
                #soa_ident::new_value(#(self.#field_idents.clone()),*)
            }
        }

        // ++++++++++++++++++++ Soa ++++++++++++++++++++
        
        #[derive(Default, Clone)]
        #vis struct #soa_ident {
            _len: usize, // TODO get rid of this?
            #(#field_idents: Vec<#field_tys>),*
        }

        impl #soa_ident {
            fn new_value(#(#field_idents: #field_tys),*) -> #ident {
                #ident{ #(#field_idents),* }
            }
            fn new_ref<'a>(#(#field_idents: &'a #field_tys),*) -> #soa_ref_ident<'a> {
                #soa_ref_ident{ #(#field_idents),* }
            }
            fn new_mut<'a>(#(#field_idents: &'a mut #field_tys),*) -> #soa_mut_ident<'a> {
                #soa_mut_ident{ #(#field_idents),* }
            }

            #vis fn new() -> Self { Self::default() }
            
            #vis fn len(&self) -> usize { self._len }
            #vis fn is_empty(&self) -> bool { self._len == 0 }

            // TODO? return avg? capacities()?
            // #vis fn capacity(&self) -> usize {
            //     let mut res = 0;
            //     #(res += self.#field_idents.capacity();)*
            //     res
            // }
            #vis fn reserve(&mut self, add: usize){
                #(self.#field_idents.reserve(add);)*
            }
            #vis fn reserve_exact(&mut self, add: usize){
                #(self.#field_idents.reserve_exact(add);)*
            }

            #vis fn push(&mut self, val: #ident){
                self._len += 1;
                #(self.#field_idents.push(val.#field_idents2);)*
            }

            #vis fn pop(&mut self) -> Option<#ident> {
                if self._len != 0 {
                    self._len -= 1;
                    Some(Self::new_value(#(self.#field_idents.pop().unwrap()),*))
                } else {
                    None
                }
            }

            // // TODO #vis fn extend_from_slices
            // // TODO #vis fn extend_from_iters

            #vis fn swap(&mut self, a: usize, b: usize){
                assert!(a < self._len);
                assert!(b < self._len);
                #(self.#field_idents.swap(a, b);)*
            }
            #vis fn swap_remove(&mut self, idx: usize) -> #ident {
                assert!(idx < self._len);
                Self::new_value(#(self.#field_idents.swap_remove(idx),)*)
            }

            #vis fn get(&self, idx: usize) -> Option<#soa_ref_ident> {
                if idx < self._len {
                    Some(Self::new_ref(#(&self.#field_idents[idx]),*))
                } else {
                    None
                }
            }
            #vis fn get_mut(&mut self, idx: usize) -> Option<#soa_mut_ident> {
                if idx < self._len {
                    Some(Self::new_mut(#(&mut self.#field_idents[idx]),*))
                } else {
                    None
                }
            }
            #vis unsafe fn get_unchecked(&self, idx: usize) -> #soa_ref_ident {
                Self::new_ref(#(self.#field_idents.get_unchecked(idx)),*)
            }
            #vis unsafe fn get_unchecked_mut(&mut self, idx: usize) -> #soa_mut_ident {
                Self::new_mut(#(self.#field_idents.get_unchecked_mut(idx)),*)
            }
            // TODO Naming? Should be contained in SoaPtrs?
            #vis unsafe fn ptr_read(&self, idx: usize) -> #ident {
                use std::ptr;

                Self::new_value(#(ptr::read(self.#field_idents.get_unchecked(idx))),*)
            }
            // TODO Naming? Should be contained in SoaPtrs?
            #vis unsafe fn ptr_write(&mut self, idx: usize, value: #ident){
                use std::ptr;

                #(ptr::write(self.#field_idents.get_unchecked_mut(idx), value.#field_idents2));*
            }

            // TODO *SoaPtrs, SoaSlices-types?
            // #vis fn as_ptrs(&self) -> (#(*const #field_tys,)*) {
            //     (#(self.#field_idents.as_ptr(),)*)
            // }
            // #vis fn as_slices(&self) -> (#(&[#field_tys],)*) {
            //     (#(self.#field_idents.as_slice(),)*)
            // }

            #vis unsafe fn set_len(&mut self, len: usize){
                self._len = len;
                #(self.#field_idents.set_len(len);)*
            }
            #vis fn clear(&mut self){
                self._len = 0;
                #(self.#field_idents.clear();)*
            }

            #vis fn iter(&self) -> #soa_iter_ident {
                #soa_iter_ident::new(self)
            }
            #vis fn iter_mut(&mut self) -> #soa_iter_mut_ident {
                #soa_iter_mut_ident::new(self)
            }
        }

        impl Extend<#ident> for #soa_ident {
            fn extend<I>(&mut self, iterable: I)
                where I: IntoIterator<Item = #ident>
            {
                // TODO correct? use ptr_write and set_len?

                let iter = iterable.into_iter();
                self.reserve(iter.size_hint().0);
                for value in iter {
                    #(self.#field_idents.push(value.#field_idents2);)*
                }
            }
        }

        impl ::std::iter::FromIterator<#ident> for #soa_ident {
            fn from_iter<I>(iterable: I) -> Self
                where I: IntoIterator<Item = #ident>
            {
                let mut ret = Self::new();
                ret.extend(iterable);
                ret
            }
        }

        impl<'a> IntoIterator for &'a #soa_ident {
            type Item = #soa_ref_ident<'a>;
            type IntoIter = #soa_iter_ident<'a>;
            fn into_iter(self) -> Self::IntoIter { self.iter() }
        }

        impl<'a> IntoIterator for &'a mut #soa_ident {
            type Item = #soa_mut_ident<'a>;
            type IntoIter = #soa_iter_mut_ident<'a>;
            fn into_iter(self) -> Self::IntoIter { self.iter_mut() }
        }

        // ++++++++++++++++++++ SoaIter ++++++++++++++++++++

        #[derive(Clone)]
        #vis struct #soa_iter_ident<'a> {
            range: ::std::ops::Range<usize>,
            soa: &'a #soa_ident,
        }

        impl<'a> #soa_iter_ident<'a> {
            fn new(soa: &'a #soa_ident) -> Self {
                let len = soa.len();
                Self{ 
                    range: ::std::ops::Range{ start: 0, end: len },
                    soa,
                }
            }
        }

        impl<'a> Iterator for #soa_iter_ident<'a> {
            type Item = #soa_ref_ident<'a>;
            fn next(&mut self) -> Option<Self::Item> {
                match self.range.next() {
                    Some(idx) => Some(unsafe { self.soa.get_unchecked(idx) }), // TODO correct?
                    None => None
                }
            }
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.range.size_hint()
            }
        }

        impl<'a> DoubleEndedIterator for #soa_iter_ident<'a> {
            fn next_back(&mut self) -> Option<Self::Item> {
                match self.range.next_back() {
                    Some(idx) => Some(unsafe { self.soa.get_unchecked(idx) }), // TODO correct?
                    None => None
                }
            }
        }

        impl<'a> ExactSizeIterator for #soa_iter_ident<'a> {
            fn len(&self) -> usize {
                self.range.len()
            }
        }

        // ++++++++++++++++++++ SoaIterMut ++++++++++++++++++++

        #vis struct #soa_iter_mut_ident<'a> {
            range: ::std::ops::Range<usize>,
            soa: &'a mut #soa_ident,
        }
       
        impl<'a> #soa_iter_mut_ident<'a> {
            fn new(soa: &'a mut #soa_ident) -> Self {
                use std::ops::Range;

                let len = soa.len();
                Self{ 
                    range: Range{ start: 0, end: len },
                    soa,
                }
            }
        }

        impl<'a> Iterator for #soa_iter_mut_ident<'a> {
            type Item = #soa_mut_ident<'a>;
            fn next(&mut self) -> Option<Self::Item> {
                use std::mem;

                match self.range.next() {
                    Some(idx) => Some(unsafe { mem::transmute(self.soa.get_unchecked_mut(idx)) }), // TODO correct?
                    None => None
                }
            }
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.range.size_hint()
            }
        }

        impl<'a> DoubleEndedIterator for #soa_iter_mut_ident<'a> {
            fn next_back(&mut self) -> Option<Self::Item> {
                use std::mem;

                match self.range.next_back() {
                    Some(idx) => Some(unsafe { mem::transmute(self.soa.get_unchecked_mut(idx)) }), // TODO correct?
                    None => None
                }
            }
        }

        impl<'a> ExactSizeIterator for #soa_iter_mut_ident<'a> {
            fn len(&self) -> usize {
                self.range.len()
            }
        } 

    }
}
