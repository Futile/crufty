#![feature(const_fn)]
#![feature(associated_consts)]
#![feature(duration_checked_ops)]

#[macro_use]
extern crate custom_derive;
#[macro_use]
extern crate newtype_derive;

#[macro_use]
extern crate glium;

#[macro_use]
extern crate ecs;

extern crate image;

extern crate nalgebra as na;
extern crate ncollide as nc;
extern crate num;

extern crate hprof;
extern crate clock_ticks;
extern crate ordered_float;
extern crate toml;

extern crate mio;
extern crate byteorder;
extern crate rand;
extern crate smallvec;

extern crate specs;

#[macro_use]
pub mod util;

pub mod application;
pub mod net;
pub mod systems;
pub mod components;
pub mod game;
