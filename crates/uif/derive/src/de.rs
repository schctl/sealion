use proc_macro2::{TokenStream, Ident};
use quote::quote;
use syn::{DeriveInput, DataStruct, Data, Fields};


pub fn impl_deserialize(input: DeriveInput) -> TokenStream {
    let name = input.ident;

    let ts = match input.data {
        Data::Struct(ds) => impl_deserialize_struct(name, ds),
        _ => panic!("only struct supported")
    };

    quote! {
        #input
        #ts
    }
}

fn impl_deserialize_struct(name: Ident, input: DataStruct) -> TokenStream {
    todo!()
}
