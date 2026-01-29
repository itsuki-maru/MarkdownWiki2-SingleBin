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
    is_edit_request: boolean;
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

interface TypeWikiOwner {
    wikiOwner: string;
    publicName: String;
    isOwner: boolean;
}

// EditRequestStatus (更新リクエストの状態)
type EditRequestStatus =
    | "REJECT"
    | "REQUESTNOW"
    | "DRAFT"
    | "APPLIED";

// バックエンドから取得するリクエスト状況
interface EditRequestWiki {
    id: string,
    wiki_owner_id: string,
    request_public_user_name: string,
    request_wiki_id: string,
    original_title: string,
    original_body: string,
    edit_request_title: string,
    edit_request_body: string,
    create_at: string,
    request_message: string | null,
    status: EditRequestStatus,
}

// 編集リクエスト
interface EditWikiRequestData {
    edit_wiki_id: string,
    edit_request_title: string,
    edit_request_body: string,
    request_message: string | null,
    status: EditRequestStatus,
}

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
    ApplicationInit,
    TypeWikiOwner,
    EditRequestStatus,
    EditWikiRequestData,
    EditRequestWiki,
}