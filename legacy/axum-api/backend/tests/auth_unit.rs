//! Unit tests for auth hashing and JWT claims (no database required).

use caseflow_cms::services::auth::AuthService;

#[test]
fn password_hash_roundtrip() {
    let hash = AuthService::hash_password("secret123").expect("hash");
    assert!(AuthService::verify_password("secret123", &hash).unwrap());
    assert!(!AuthService::verify_password("wrong", &hash).unwrap());
}

#[test]
fn refresh_token_hash_is_stable() {
    let a = AuthService::hash_token("rt_abc");
    let b = AuthService::hash_token("rt_abc");
    let c = AuthService::hash_token("rt_xyz");
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(a.len(), 64);
}
