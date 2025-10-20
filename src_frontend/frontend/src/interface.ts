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

interface OneTimeWikis {
    id: string;
    user_id: string;
    wiki_id: string;
    url: string;
    expiration: string;
    title: string;
    create_at: string;
}

interface LocalStrageItem {
    isShowToolsFromLocalStrage: string | null;
    isPreviewFromLocalStrage: string | null;
};

// アプリケーションの起動時情報
interface ApplicationInit {
    appTitle: string,
    allowUserAccountCreate: boolean,
    allowOrigins: string, // ex) http://localhost:3080,www.example.com
};


export type {
    LoginUser,
    CreateWikiData,
    WikiData,
    UpdateWikiData,
    deleteWikiData,
    QueryForm,
    ImageData,
    OneTimeWikis,
    LocalStrageItem,
    ApplicationInit
}