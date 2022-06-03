use core::ops::Deref;
use std::mem::MaybeUninit;

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! nicks_static_internal {
    ($(#[$attr:meta])* ($($vis:tt)*) static ref $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        nicks_static_internal!(@MAKE TY, $(#[$attr])*, ($($vis)*), $N);
        nicks_static_internal!(@TAIL, $N : $T = $e);
        nicks_static!($($t)*);
    };
    (@TAIL, $N:ident : $T:ty = $e:expr) => {
        impl Deref for $N {
            type Target = $T;
            fn deref(&self) -> &$T {
                #[inline(always)]
                fn __static_ref_initialize() -> $T { $e }

				static mut RES: MaybeUninit<$T> = MaybeUninit::uninit();
				unsafe {
					RES.write(__static_ref_initialize());
					RES.assume_init_ref()
				}
            }
        }
    };
    (@MAKE TY, $(#[$attr:meta])*, ($($vis:tt)*), $N:ident) => {
        #[allow(missing_copy_implementations)]
        #[allow(non_camel_case_types)]
        #[allow(dead_code)]
        $(#[$attr])*
        $($vis)* struct $N {__private_field: ()}
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        $($vis)* static $N: $N = $N {__private_field: ()};
    };
    () => ()
}

#[macro_export(local_inner_macros)]
macro_rules! nicks_static {
    ($(#[$attr:meta])* static ref $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        nicks_static_internal!($(#[$attr])* () static ref $N : $T = $e; $($t)*);
    };
    ($(#[$attr:meta])* pub static ref $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        nicks_static_internal!($(#[$attr])* (pub) static ref $N : $T = $e; $($t)*);
    };
    ($(#[$attr:meta])* pub ($($vis:tt)+) static ref $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        nicks_static_internal!($(#[$attr])* (pub ($($vis)+)) static ref $N : $T = $e; $($t)*);
    };
    () => ()
}

#[cfg(debug_assertions)]
macro_rules! shader {
    ($name:ident => $filename:literal) => {
        nicks_static! {
            pub static ref $name: String =
                std::fs::read_to_string($filename).expect("Unable to open shader file");
        }
    };
    () => {};
}

use include_flate::flate;

#[cfg(not(debug_assertions))]
macro_rules! shader {
    ($name:ident => $filename:literal) => {
        flate!(pub static $name: str from $filename);
    };
    () => {};
}

shader!(GEOMETRY => "src/assets/shaders/geometry.wgsl");
shader!(PARTICLE => "src/assets/shaders/particle.wgsl");
shader!(SHADE_DIRECT => "src/assets/shaders/shade_direct.wgsl");
shader!(SHADOW => "src/assets/shaders/shadow.wgsl");
shader!(SKYBOX => "src/assets/shaders/skybox.wgsl");
shader!(UI => "src/assets/shaders/ui.wgsl");

// bloom stuff
shader!(DOWNSAMPLE_BLOOM => "src/assets/shaders/downsample_bloom.wgsl");
shader!(KAWASE_BLUR_DOWN => "src/assets/shaders/kawase_blur_down.wgsl");
shader!(KAWASE_BLUR_UP => "src/assets/shaders/kawase_blur_up.wgsl");
shader!(COMPOSITE_BLOOM => "src/assets/shaders/composite_bloom.wgsl");

// hbil stuff (more simillar to hbao for now)
shader!(HBIL => "src/assets/shaders/hbil.wgsl");
shader!(HBIL_DEBAYER => "src/assets/shaders/hbil_debayer.wgsl");

// probes (unused for now)
shader!(INIT_PROBES => "src/assets/shaders/init_probes.wgsl");
shader!(GEOMETRY_ACC_PROBES => "src/assets/shaders/geometry_acc_probes.wgsl");
shader!(TEMPORAL_ACC_PROBES => "src/assets/shaders/geometry_acc_probes.wgsl");

// util
shader!(WRONSKI_AA => "src/assets/shaders/wronski_aa.wgsl");
shader!(DOWNSAMPLE_MITCHELL => "src/assets/shaders/downsample_mitchell.wgsl");
shader!(SIMPLE_FSQ => "src/assets/shaders/simple_fsq.wgsl");
shader!(SURFEL_GEOMETRY => "src/assets/shaders/surfel_geometry.wgsl");
