<div align="center">

[ [English](README) | 日本語 ]

<br>

<img alt="home" width="40%" src="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/mnemnk_title.png?raw=true">

<br>

![Badge Workflow]
![Badge Language] 
[![Badge License]][License] 

<br>

Mnemnkは、あなたの活動を記録し、常時稼働するマルチエージェントシステムを通じてそれらを強化する、パーソナルなライフログプラットフォームです。

<br>

<a target="_blank" href="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-home.png?raw=true"><img alt="home" width="60%" src="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-home.png?raw=true"></a>

</div>

## 特徴

### アクティビティの自動保存

- アプリケーションの使用履歴やブラウザ履歴がスクリーンショットと共に自動的に保存されます。
- あなたの日々のデジタルフットプリントが時系列で整理され、そのときのスクリーンショットと共に振り返ることができます。
- 保存された情報は日付で確認したり、テキスト検索で探すことができます。

<br>
<div align="center">
<a target="_blank" href="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-daily.png?raw=true"><img alt="daily" width="60%" src="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-daily.png?raw=true"></a>
</div>

### プライバシー重視

- このコアシステム `mnemnk-app` とコアエージェントによって記録された活動記録はローカルに保存され、外部に送信されることはありません。
- コアシステムとコアエージェントはオープンソースソフトウェアとして公開されており、透明性が確保されています。

<br>
<div align="center">
<a target="_blank" href="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-core-agents.png?raw=true"><img alt="core agents" width="60%" src="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-core-agents.png?raw=true"></a>
</div>

### エージェントベースの拡張性

- システムの拡張性は、さまざまな機能をエージェントとして分離することで実現されています。
- エージェントは任意のプログラミング言語で開発可能で、自由なシステム拡張が可能です。
- フローを用いた視覚的なUIにより、多数のエージェントが協調動作するマルチエージェントシステムを直観的に構築できます。
- 一度限りのバッチ処理システムとは異なり、Mnemnkのエージェントはリアルタイムでイベントを処理しながら並行して動作します。

## インストール

[リリース](https://github.com/mnemnk/mnemnk-app/releases)からインストーラーをダウンロードし実行する。

<details>
  <summary><strong>リポジトリーからのビルド</strong></summary>

### 開発

あなたが開発者なら、リポジトリーからビルドすることも可能です。

### 事前に必要なもの

[Tauri](https://v2.tauri.app/)の開発環境が必要です。
- Git
- [Rust](https://www.rust-lang.org/)
- [npm](https://nodejs.org/ja/)

### ビルド

```shell
git clone https://github.com/mnemnk/mnemnk-app.git
cd mnemnk-app
npm install
npm run tauri:dev
```

(`npm run tauri dev` でも動きますが、その場合はidentifierがrelease buildと同じになります)

</details>

### 設定

はじめて起動するとMnemnk Directoryを指定するようにSettingsページが開きます。ディスク容量が十分にある場所を指定してください。Databaseのファイルも作成されますので、クラウド同機が行われる場所は避けた方がいいでしょう。

設定したらSaveをクリックし、アプリを再起動してください。

<br>
<div align="center">
<a target="_blank" href="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-settings.png?raw=true"><img alt="settings" width="60%" src="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-settings.png?raw=true"></a>
</div>
<br>

## Agents

各エージェントをそれぞれのページからインストールしてください：

- [mnemnk-core-agents](https://github.com/mnemnk/mnemnk-core-agents)
  - 以下の基本的なエージェントが含まれます。
  - [mnemnk-api](https://github.com/mnemnk/mnemnk-core-agents/tree/main/mnemnk-api)
  - [mnemnk-application](https://github.com/mnemnk/mnemnk-core-agents/tree/main/mnemnk-application)
  - [mnemnk-screen](https://github.com/mnemnk/mnemnk-core-agents/tree/main/mnemnk-screen)
- [mnemnk-langchain](https://github.com/mnemnk/mnemnk-langchain)
  - [LangChain](https://www.langchain.com/langchain) を用いたエージェント
- [mnemnk-browser-extension](https://github.com/mnemnk/mnemnk-browser-extension)
  - `mnemnk-api` と通信し、ブラウザ履歴を保存するためのブラウザ拡張

## Contribution

- エージェント開発：新しいエージェントを作成して機能を拡張
- バグ報告：問題の特定と修正を支援
- 機能リクエスト：新しい機能のアイデアを提供

## Contact

Akira Ishino - [stn](https://github.com/stn) - akira@lumilab.jp

## Acknowledgments

* [Rust](https://www.rust-lang.org/)
* [Tauri](https://tauri.app/)
* [Tokio](https://tokio.rs/)
* [SurrealDB](https://surrealdb.com/)
* [Node.js](https://nodejs.org/)
* [TypeScript](https://www.typescriptlang.org/)
* [Svelte](https://svelte.dev/)
* [Svelte Flow](https://svelteflow.dev/)
* [Tailwind CSS](https://tailwindcss.com/)
* [Flowbite Svelte](https://flowbite-svelte.com/)

<!----------------------------------------------------------------------------->

[License]: LICENSE

<!----------------------------------{ Badges }--------------------------------->

[Badge Workflow]: https://github.com/mnemnk/mnemnk-app/actions/workflows/publish.yml/badge.svg
[Badge Language]: https://img.shields.io/github/languages/top/mnemnk/mnemnk-app
[Badge License]: https://img.shields.io/github/license/mnemnk/mnemnk-app
