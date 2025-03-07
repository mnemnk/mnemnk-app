<div align="center">

[ [English](README) | 日本語 ]

<br>

![Mnemnk logo](docs/img/mnemnk_title.png)

<br>

![Badge Workflow]
![Badge Language] 
[![Badge License]][License] 

<br>

Mnemnkは、あなたの活動を記録し、マルチエージェントによって強化するパーソナルなライフロギングプラットフォームです。

<br>

<a target="_blank" href="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-home.png?raw=true"><img alt="home" width="60%" src="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-home.png?raw=true"></a>

</div>

## 特徴

### アクティビティの自動保存

- アプリケーションの使用履歴やブラウザの履歴がスクリーンショットと共に自動保存されます。
- 保存した情報は時系列で見返すことも、検索することもできます。

<br>
<div align="center">
<a target="_blank" href="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-daily.png?raw=true"><img alt="daily" width="60%" src="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-daily.png?raw=true"></a>
</div>

### プライバシー

- コアシステムmnemnk-appおよびコアエージェントによって記録される活動記録はローカルに保存され、外部に送られることはありません。
- コアシステムおよびコアエージェントはOSSにて公開されています。


### 拡張性

- 各種機能をエージェントとして分離することで、システムの拡張性を実現
- エージェントは任意の言語で開発可能


## インストール

[リリース](https://github.com/mnemnk/mnemnk-app/releases)からインストーラーをダウンロードし実行する。


### コアエージェント

各エージェントのページからインストールしてください。

- [mnemnk-application](https://github.com/mnemnk/mnemnk-application): アプリケーションの使用履歴を保存
- [mnemnk-screen](https://github.com/mnemnk/mnemnk-screen): スクリーンショットを保存
- [mnemnk-api](https://github.com/mnemnk/mnemnk-api): APIサーバーを提供
- [mnemnk-browser-extension](https://github.com/mnemnk/mnemnk-browser-extension): `mnemnk-api`と通信し、ブラウザの履歴を保存



## 設定

coreの設定はそのままでも使い始めることができます。

ディスクを複数持っているマシンの場合、SettingsページからData Directoryを指定することで、DBとスクリーンショットの保存場所を変更できます。データは自動では移動しないので、Quitしてデータをコピーしてから再起動してください。

<br>
<div align="center">
<a target="_blank" href="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-settings.png?raw=true"><img alt="settings" width="60%" src="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-settings.png?raw=true"></a>
</div>
<br>

エージェントはエージェントの設定からONにできます。


## コントリビューション

- エージェントの開発
- バグ報告
- ドキュメントの改善

<!----------------------------------------------------------------------------->

[License]: LICENSE

<!----------------------------------{ Badges }--------------------------------->

[Badge Workflow]: https://github.com/mnemnk/mnemnk-app/actions/workflows/publish.yml/badge.svg
[Badge Language]: https://img.shields.io/github/languages/top/mnemnk/mnemnk-app
[Badge License]: https://img.shields.io/github/license/mnemnk/mnemnk-app
