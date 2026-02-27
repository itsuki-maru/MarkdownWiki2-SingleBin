/**
 * src/templates/ の preview.html / preview-mobile.html が参照する
 * 外部JSおよびCSSをnode_modulesからdist/assets/へコピーするスクリプト。
 * npm run build の後に実行される。
 */

const { copyFileSync, mkdirSync, cpSync } = require('node:fs');
const { resolve, basename } = require('node:path');

const nm = (...parts) => resolve(__dirname, '../node_modules', ...parts);
const dest = (...parts) => resolve(__dirname, '../dist/assets', ...parts);

mkdirSync(dest(), { recursive: true });

function cp(src, destName) {
    const d = dest(destName ?? basename(src));
    copyFileSync(nm(src), d);
    console.log(`  ${src} → dist/assets/${destName ?? basename(src)}`);
}

console.log('Copying preview assets from node_modules...');

// marked (v17以降はmarked.min.jsが廃止されたためUMDビルドを代替として配置)
cp('marked/lib/marked.umd.js', 'marked.min.js');

// xss
cp('xss/dist/xss.min.js', 'xss.min.js');

// mermaid
cp('mermaid/dist/mermaid.min.js', 'mermaid.min.js');

// prismjs (コアとテーマ)
cp('prismjs/prism.js', 'prism.js');
cp('prismjs/themes/prism-okaidia.css', 'prism-okaidia.css');

// prismjs 言語コンポーネント
for (const lang of [
    'typescript', 'javascript', 'bash', 'python', 'rust',
    'markup', 'json', 'markdown', 'powershell', 'sql',
    'toml', 'yaml', 'uri', 'c', 'docker',
]) {
    cp(`prismjs/components/prism-${lang}.js`, `prism-${lang}.js`);
}

// katex
cp('katex/dist/katex.min.js', 'katex.min.js');
cp('katex/dist/katex.min.css', 'katex.min.css');
cp('katex/dist/contrib/auto-render.min.js', 'auto-render.min.js');

// katex fonts (katex.min.css が ./fonts/ を相対パスで参照するため一緒にコピー)
mkdirSync(dest('fonts'), { recursive: true });
cpSync(nm('katex/dist/fonts'), dest('fonts'), { recursive: true });

// GitHub Markdown CSS (github.css として配置)
cp('github-markdown-css/github-markdown.css', 'github.css');

console.log('Done.');
