pub mod pubkey;
pub mod status;
pub mod token;
pub mod user;

pub use pubkey::{AuthPubkeyContext, AuthPubkeyRoute, RSAKey};
pub use status::{AuthStatusContext, AuthStatusRoute};
pub use token::{AuthTokenContext, AuthTokenRoute};
pub use user::{AuthUserContext, AuthUserRoute};

const PASSWORD_SALT: [u8; 64] = [
    14, 169, 19, 53, 220, 112, 183, 235, 112, 165, 131, 132, 68, 29, 167, 65, 150, 219, 121, 212,
    121, 47, 132, 195, 216, 119, 172, 134, 208, 11, 2, 80, 105, 176, 45, 194, 78, 84, 16, 169, 228,
    25, 195, 207, 144, 204, 171, 95, 8, 113, 93, 40, 41, 116, 80, 126, 253, 142, 245, 147, 148,
    136, 121, 220,
];
const PASSWORD_ITER: usize = 10000;
