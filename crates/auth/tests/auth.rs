use std::time::Duration;

use openwebhmi_auth::{
    can_write_in_view, Permission, Role, SessionManager, UserPatch, UserStore, ViewAcl,
};

#[test]
fn bcrypt_round_trip_accepts_right_password_and_rejects_wrong_password() {
    let store = UserStore::memory().unwrap();
    store
        .create_user(
            "alice",
            "correct horse battery staple",
            vec![Role::Operator],
        )
        .unwrap();

    assert!(store
        .authenticate("alice", "correct horse battery staple")
        .unwrap()
        .is_some());
    assert!(store.authenticate("alice", "wrong").unwrap().is_none());
}

#[test]
fn jwt_issue_verify_rejects_tampered_and_expired_tokens() {
    let manager = SessionManager::new(b"secret".to_vec(), Duration::from_secs(60));
    let token = manager
        .issue("user-1", "alice", &[Role::Administrator])
        .unwrap();
    let session = manager.verify(&token).unwrap();
    assert_eq!(session.user_id, "user-1");
    assert_eq!(session.roles, vec![Role::Administrator]);

    let mut tampered = token.clone();
    tampered.push('x');
    assert!(manager.verify(&tampered).is_err());

    let expired = SessionManager::new(b"secret".to_vec(), Duration::from_secs(0))
        .issue("user-1", "alice", &[Role::Viewer])
        .unwrap();
    std::thread::sleep(Duration::from_secs(1));
    assert!(manager.verify(&expired).is_err());
}

#[test]
fn acl_operator_write_defaults_to_allowed_but_can_be_restricted() {
    assert!(Role::Operator.allows(Permission::WriteTags));
    assert!(can_write_in_view(&[Role::Operator], None));
    assert!(!can_write_in_view(
        &[Role::Operator],
        Some(&ViewAcl {
            allowed_roles: Some(vec![Role::Designer])
        })
    ));
    assert!(can_write_in_view(
        &[Role::Designer],
        Some(&ViewAcl {
            allowed_roles: Some(vec![Role::Designer])
        })
    ));
}

#[test]
fn bootstrap_admin_creates_once_and_upsert_preserves_existing_without_password() {
    let store = UserStore::memory().unwrap();
    let admin = store.bootstrap_admin(Some("admin-pass")).unwrap();
    assert_eq!(admin.user.username, "admin");
    assert_eq!(admin.generated_password, None);

    let updated = store
        .upsert_user(UserPatch {
            username: "admin".into(),
            password: None,
            roles: vec![Role::Administrator, Role::Designer],
        })
        .unwrap();
    assert_eq!(updated.roles, vec![Role::Administrator, Role::Designer]);
    assert!(store
        .bootstrap_admin(None)
        .unwrap()
        .generated_password
        .is_none());
}
