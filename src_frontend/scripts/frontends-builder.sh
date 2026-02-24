#!/usr/bin/bash

# 依存関係を更新するかのフラグオプション`-d`を引数で受け取る
DEPENDS_DELETE_FLAG=false

while getopts "d" opt; do
  case $opt in
    d)
      DEPENDS_DELETE_FLAG=true
      ;;
    *)
      echo "Usage: $0 [-d]"
      exit 1
      ;;
  esac
done

# このスクリプトファイルのディレクトリを取得
scriptDir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# src_frontend を取得
projectDir="$(dirname "$scriptDir")"

# プロジェクトルート
projectRoot="$(dirname "$projectDir")"

# frontendディレクトリ
frontendDir="$projectDir/frontend"
frontendDistDir="$frontendDir/dist"

# frontend-mobileディレクトリ
frontendMobileDir="$projectDir/frontend-mobile"
frontendMobileDistDir="$frontendMobileDir/dist"

# frontend-adminディレクトリ
frontendAdminDir="$projectDir/frontend-admin"
frontendAdminDistDir="$frontendAdminDir/dist"

# templates ディレクトリ
rustTemplatesDir="$projectRoot/src/templates"

# 過去のビルドファイル
distributionDirOld="$projectRoot/dist"

# mainディレクトリ
mainDir="$projectDir/main"
mainDistDir="$mainDir/dist"
mainDistAssetsDir="$mainDir/dist/assets"

# 移動対象のファイル
faviconDir="$mainDir/dist/favicon.ico"
pngFiles="$mainDir/dist/*.png"
jsFiles="$mainDir/dist/*.js"
mjsFiles="$mainDir/dist/*.mjs"
cssFiles="$mainDir/dist/*.css"
jsonFiles="$mainDir/dist/*.json"
cmapsFiles="$mainDir/dist/cmaps/*"
movedDir="$mainDir/dist/assets/"


# 前回のビルドファイルが存在する場合は削除
if [ -d "$mainDistDir" ]; then
    rm -rf "$mainDistDir"
    echo "Directory '$mainDistDir' has been removed."
else
    echo "Directory '$mainDistDir' does not exist."
fi

if [ -d "$frontendDistDir" ]; then
    rm -rf "$frontendDistDir"
    echo "Directory '$frontendDistDir' has been removed."
else
    echo "Directory '$frontendDistDir' does not exist."
fi

if [ -d "$frontendMobileDistDir" ]; then
    rm -rf "$frontendMobileDistDir"
    echo "Directory '$frontendMobileDistDir' has been removed."
else
    echo "Directory '$frontendMobileDistDir' does not exist."
fi

if [ -d "$frontendAdminDistDir" ]; then
    rm -rf "$frontendAdminDistDir"
else
    echo "Directory '$frontendAdminDistDir' does not exist."
fi

if [ -d "$distributionDirOld" ]; then
    rm -rf "$distributionDirOld"
    echo "Directory '$distributionDirOld' has been removed."
else
    echo "Directory '$distributionDirOld' does not exist."
fi

# frontendの処理
cd $frontendDir

nodeModules="$frontendDir/node_modules"

# -dオプション設定時にはnode_modulesを削除
if [ "$DEPENDS_DELETE_FLAG" = true ]; then
    rm -rf "$nodeModules"
    echo "Directory '$nodeModules' has been removed."
else
    echo "Directory '$TARGET_DIR' exists, but -d option was not provided. No action taken."
fi

# node_modulesが存在しなければnpm installを実行
if [ ! -d "$nodeModules" ]; then
    npm install
else
    echo "Directory '$nodeModules' exist."
fi

# ビルド
npm run build
# HTMLファイルパス
targetHtml="$frontendDir/dist/index.html"

# favicon.icoのパス変更
sed -i 's|href="./favicon.ico"|href="/assets/favicon.ico"|g' "$targetHtml"
# qrcode.min.jsのパス変更
sed -i 's|src="./qrcode.min.js"|src="/assets/qrcode.min.js"|g' "$targetHtml"
# diff_match_patch.jsのパス変更
sed -i 's|src="./diff_match_patch.js"|src="/assets/diff_match_patch.js"|g' "$targetHtml"
# mermaid.min.jsのパス変更
sed -i 's|src="./mermaid.min.js"|src="/assets/mermaid.min.js"|g' "$targetHtml"
# manifest-tab.jsonのパス変更
sed -i 's|href="./manifest-tab.json"|href="/assets/manifest-tab.json"|g' "$targetHtml"
# apple-touch-icon.pngのパス変更
sed -i 's|href="./apple-touch-icon.png"|href="/assets/apple-touch-icon.png"|g' "$targetHtml"


# frontend-mobileの処理
cd $frontendMobileDir

nodeModules="$frontendMobileDir/node_modules"

# -dオプション設定時にはnode_modulesを削除
if [ "$DEPENDS_DELETE_FLAG" = true ]; then
    rm -rf "$nodeModules"
    echo "Directory '$nodeModules' has been removed."
else
    echo "Directory '$TARGET_DIR' exists, but -d option was not provided. No action taken."
fi

# node_modulesが存在しなければnpm installを実行
if [ ! -d "$nodeModules" ]; then
    npm install
else
    echo "Directory '$nodeModules' exist."
fi

# ビルド
npm run build
# HTMLファイルパス
targetHtml="$frontendMobileDir/dist/index.html"

# favicon.icoのパス変更
sed -i 's|href="./favicon.ico"|href="/assets/favicon.ico"|g' "$targetHtml"
# qrcode.min.jsのパス変更
sed -i 's|src="./qrcode.min.js"|src="/assets/qrcode.min.js"|g' "$targetHtml"
# diff_match_patch.jsのパス変更
sed -i 's|src="./diff_match_patch.js"|src="/assets/diff_match_patch.js"|g' "$targetHtml"
# mermaid.min.jsのパス変更
sed -i 's|src="./mermaid.min.js"|src="/assets/mermaid.min.js"|g' "$targetHtml"
# manifest.jsonのパス変更
sed -i 's|href="./manifest.json"|href="/assets/manifest.json"|g' "$targetHtml"
# apple-touch-icon.pngのパス変更
sed -i 's|href="./apple-touch-icon.png"|href="/assets/apple-touch-icon.png"|g' "$targetHtml"

targetHtmlNewName="$frontendMobileDir/dist/index-mobile.html"
mv $targetHtml $targetHtmlNewName


# frontend-adminの処理
cd $frontendAdminDir

nodeModules="$frontendAdminDir/node_modules"

# -dオプション設定時にはnode_modulesを削除
if [ "$DEPENDS_DELETE_FLAG" = true ]; then
    rm -rf "$nodeModules"
    echo "Directory '$nodeModules' has been removed."
else
    echo "Directory '$TARGET_DIR' exists, but -d option was not provided. No action taken."
fi

# node_modulesが存在しなければnpm installを実行
if [ ! -d "$nodeModules" ]; then
    npm install
else
    echo "Directory '$nodeModules' exist."
fi

# ビルド
npm run build

# HTMLファイルパス
targetHtml="$frontendAdminDir/dist/index.html"

# favicon.icoのパス変更
sed -i 's|href="./favicon.ico"|href="/assets/favicon.ico"|g' "$targetHtml"

targetHtmlNewName="$frontendAdminDir/dist/index-admin.html"
mv $targetHtml $targetHtmlNewName


# mainの処理
cd $mainDir
# frontend/distとfrontend-mobile/dist、frontend-admin/dist配下のファイルをmainディレクトリにコピー
cp -r $frontendDistDir ./
cp -r $frontendMobileDistDir ./
cp -r $frontendAdminDistDir ./

# 静的ファイルを配置移動
mv $faviconDir $mainDistAssetsDir
mv $pngFiles $mainDistAssetsDir
mv $jsFiles $mainDistAssetsDir
mv $mjsFiles $mainDistAssetsDir
mv $cssFiles $mainDistAssetsDir
mv $jsonFiles $mainDistAssetsDir
mv $cmapsFiles $mainDistAssetsDir

# フロントエンド成果物配布用ディレクトリ作成
cd $mainDistDir
cp -r $mainDistDir $projectRoot
cp -r $rustTemplatesDir "$projectRoot/dist"
