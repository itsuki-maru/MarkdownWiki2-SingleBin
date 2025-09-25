interface LoginUser {
    username?: string;
    isAuthenticated: boolean;
}

interface UserData {
    id: string;
    username: string;
    password: string;
    create_at: string;
    is_superuser: boolean;
    is_locked: boolean;
}

interface LockedUserData {
    id: string;
    username: string;
    is_locked: boolean;
}

interface UpdateUserData {
    id: string;
    username: string;
    new_password: string;
    is_superuser: boolean;
}

export type {LoginUser, UserData, UpdateUserData, LockedUserData}