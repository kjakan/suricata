/* Copyright (C) 2020 Open Information Security Foundation
 *
 * You can copy, redistribute or modify this Program under the terms of
 * the GNU General Public License version 2 as published by the Free
 * Software Foundation.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * version 2 along with this program; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA
 * 02110-1301, USA.
 */

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{self, parse_macro_input, DeriveInput};

pub fn derive_app_layer_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let mut fields = Vec::new();
    let mut vals = Vec::new();
    let mut cstrings = Vec::new();
    let mut names = Vec::new();

    match input.data {
        syn::Data::Enum(ref data) => {
            for (i, v) in (&data.variants).into_iter().enumerate() {
                fields.push(v.ident.clone());
                let name = transform_name(&v.ident.to_string());
                let cname = format!("{}\0", name);
                names.push(name);
                cstrings.push(cname);
                vals.push(i as i32);
            }
        }
        _ => panic!("AppLayerEvent can only be derived for enums"),
    }

    let expanded = quote! {
        impl crate::applayer::AppLayerEvent for #name {
            fn from_id(id: i32) -> Option<#name> {
                match id {
                    #( #vals => Some(#name::#fields) ,)*
                    _ => None,
                }
            }

            fn as_i32(&self) -> i32 {
                match *self {
                    #( #name::#fields => #vals ,)*
                }
            }

            fn to_cstring(&self) -> &str {
                match *self {
                    #( #name::#fields => #cstrings ,)*
                }
            }

            fn from_string(s: &str) -> Option<#name> {
                match s {
                    #( #names => Some(#name::#fields) ,)*
                    _ => None
                }
            }

            unsafe extern "C" fn get_event_info(
                event_name: *const std::os::raw::c_char,
                event_id: *mut std::os::raw::c_int,
                event_type: *mut crate::core::AppLayerEventType,
            ) -> std::os::raw::c_int {
                crate::applayer::get_event_info::<#name>(event_name, event_id, event_type)
            }

            unsafe extern "C" fn get_event_info_by_id(
                event_id: std::os::raw::c_int,
                event_name: *mut *const std::os::raw::c_char,
                event_type: *mut crate::core::AppLayerEventType,
            ) -> i8 {
                crate::applayer::get_event_info_by_id::<#name>(event_id, event_name, event_type)
            }

        }
    };

    proc_macro::TokenStream::from(expanded)
}

/// Transform names such as "OneTwoThree" to "one_two_three".
pub fn transform_name(in_name: &str) -> String {
    let mut out = String::new();
    for (i, c) in in_name.chars().enumerate() {
        if i == 0 {
            out.push_str(&c.to_lowercase().to_string());
        } else if c.is_uppercase() {
            out.push('_');
            out.push_str(&c.to_lowercase().to_string());
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_transform_name() {
        assert_eq!(transform_name("One"), "one".to_string());
        assert_eq!(transform_name("SomeEvent"), "some_event".to_string());
        assert_eq!(
            transform_name("UnassignedMsgType"),
            "unassigned_msg_type".to_string()
        );
    }
}
