// To have generic async functionalities
#![feature(async_fn_in_trait)]
// To support specifying Send bounds on async functions
#![feature(return_position_impl_trait_in_trait)]
// For circuit functions to generically operate on the number of input/output wires
#![feature(generic_const_exprs)]
// While prototyping
//#![allow(unused_variables)]
//#![allow(unreachable_code)]
//#![allow(dead_code)]
#![feature(new_uninit)]
// For thiserror to support backtraces
#![feature(error_generic_member_access)]
#![feature(test)]
extern crate test;

//extern crate crossbeam;
//extern crate num_bigint;
extern crate rand;
//extern crate rayon;
extern crate serde;
extern crate serde_json;
extern crate sha2;
extern crate tokio;

mod ffi;

pub mod auth_bits;
pub mod circuits;
pub mod ff2_128;
pub mod field;
pub mod multibuf;
pub mod party;
pub mod polynomial;
pub mod rr2_128;
pub mod utils;

pub mod base_func;
pub mod func_abit;
pub mod func_com;
pub mod func_cote;
pub mod func_mpc;
pub mod func_mult;
pub mod func_net;
pub mod func_rand;
pub mod func_thresh;
pub mod func_thresh_abit;

pub mod common_protos;
