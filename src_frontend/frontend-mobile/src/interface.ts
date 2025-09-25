interface LoginUser {
    username?: string;
    isAuthenticated: boolean;
}

interface CreateWikiData {
    title: string;
    body: string;
    is_public: boolean;
}

interface UpdateWikiData {
    id: string;
    title: string;
    body: string;
    is_public: boolean;
}

interface deleteWikiData {
    id: string;
    title: string;
    body: string;
    is_public: boolean;
}

interface WikiData {
    id: string;
    user_id: string;
    date: string;
    title: string;
    body: string;
    update_at: string;
    is_public: boolean;
}

interface QueryForm {
    query1: string;
    query2: string;
}

interface ImageData {
    id: string;
    user_id: string;
    filename: string;
    uuid_filename: string;
}

export type {LoginUser, CreateWikiData, WikiData, UpdateWikiData, deleteWikiData, QueryForm, ImageData}