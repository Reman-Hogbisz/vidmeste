use crate::models::User;

pub fn user_is_admin<T: TryInto<User>>(user_id: T) -> bool {
    let user: User = match user_id.try_into() {
        Ok(user) => user,
        Err(_) => {
            info!("Failed to convert user_id to User");
            return false;
        }
    };

    if !user.permissions.contains(&1) {
        info!(
            "User {} does not have permission to get all users",
            user.user_id
        );
        return false;
    }
    true
}
