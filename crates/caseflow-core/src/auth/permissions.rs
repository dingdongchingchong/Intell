use crate::error::{AppError, AppResult};
use crate::models::UserRole;

#[derive(Debug, Clone, Copy)]
pub enum Permission {
    CaseCreate,
    CaseRead,
    CaseUpdate,
    CaseDelete,
    CaseAssign,
    CaseExport,
    UserCreate,
    UserRead,
    UserUpdate,
    UserDelete,
    AdminAll,
}

pub fn authorize(role: &str, permission: Permission) -> AppResult<()> {
    let role = UserRole::parse(role).ok_or_else(|| AppError::Forbidden("unknown role".into()))?;
    if can_access(role, permission) {
        Ok(())
    } else {
        Err(AppError::Forbidden("insufficient permissions".into()))
    }
}

pub fn can_access(role: UserRole, permission: Permission) -> bool {
    match role {
        UserRole::Admin => true,
        UserRole::Manager => matches!(
            permission,
            Permission::CaseCreate
                | Permission::CaseRead
                | Permission::CaseUpdate
                | Permission::CaseAssign
                | Permission::CaseExport
                | Permission::UserRead
        ),
        UserRole::Investigator => matches!(
            permission,
            Permission::CaseRead | Permission::CaseUpdate | Permission::CaseCreate
        ),
        UserRole::Viewer => matches!(permission, Permission::CaseRead),
    }
}
