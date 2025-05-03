<div align="center">

<img alt="home" width="40%" src="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/mnemnk_title.png?raw=true">

<br>
<br>

![Badge Workflow]
![Badge Language] 
[![Badge License]][License] 

<br>

Mnemnk is a personal lifelogging platform that records your activities and enhances them through a continuously running multi-agent system.

</div>

## Features

### Automatic Activity Saving

- Application usage history and browser history are automatically saved along with screenshots.
- Your daily digital footprint is organized chronologically and can be reviewed alongside screenshots from that time.
- Saved information can be reviewed by date or searched using text-based search.

<br>
<div align="center">
<a target="_blank" href="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-daily.png?raw=true"><img alt="daily" width="60%" src="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-daily.png?raw=true"></a>
</div>

### Privacy-Focused

- Activity logs recorded by the core system `mnemnk-app` and lifelogging agents are stored locally and are never sent externally.
- The core system and lifelogging agents are open-source software, ensuring transparency.

<br>
<div align="center">
<a target="_blank" href="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-lifelogging-agents.png?raw=true"><img alt="lifelogging agents" width="60%" src="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-lifelogging-agents.png?raw=true"></a>
</div>

### Agent-Based Extensibility

- System extensibility is achieved by separating various functions as agents.
- Agents can be developed in any programming language, allowing for flexible system expansion.
- Using a visual flow-based UI, you can intuitively build a multi-agent system where multiple agents work together.
- Unlike one-time batch processing systems, Mnemnk's agents operate in parallel, processing events in real-time.

<br>
<div align="center">
<a target="_blank" href="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-agents.png?raw=true"><img alt="agent flow" width="60%" src="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-agents.png?raw=true"></a>
</div>

## Getting Started

Follow the instructions on the [Getting Started](https://mnemnk.com/guide/getting-started) page to complete the installation.

<details>
  <summary><strong>Build from the repo</strong></summary>

### Development

If you are a developer, you can also build the application from the repository.

### Prerequisites

You need a development environment for [Tauri](https://v2.tauri.app/):
- Git
- [Rust](https://www.rust-lang.org/)
- [npm](https://nodejs.org/)

### Build

```shell
git clone https://github.com/mnemnk/mnemnk-app.git
cd mnemnk-app
npm install
npm run tauri:dev
```

(You can also use `npm run tauri dev`, but in that case, the identifier will be the same as the release build.)

</details>

### Configuration

When you first launch the application, the Settings page will open, prompting you to specify the Mnemnk Directory. Please choose a location with sufficient disk space. Since database files will also be created, it is recommended to avoid locations that are synchronized with cloud storage.

Click "Save" after configuring, and restart the application.

<br>
<div align="center">
<a target="_blank" href="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-settings.png?raw=true"><img alt="settings" width="60%" src="https://github.com/mnemnk/mnemnk-app/blob/main/docs/img/screenshot-settings.png?raw=true"></a>
</div>
<br>

## Agents

Install each agent from their respective pages:

- [mnemnk-lifelogging-agents](https://github.com/mnemnk/mnemnk-lifelogging-agents)
  - Includes the following essential agents.
  - [mnemnk-application](https://github.com/mnemnk/mnemnk-lifelogging-agents/tree/main/mnemnk-application)
  - [mnemnk-screen](https://github.com/mnemnk/mnemnk-lifelogging-agents/tree/main/mnemnk-screen)
- [mnemnk-langchain](https://github.com/mnemnk/mnemnk-langchain)
  - Agents based on [LangChain](https://www.langchain.com/langchain).

- [mnemnk-browser-extension](https://github.com/mnemnk/mnemnk-browser-extension)
  - A browser extension that communicates with `mnemnk-api` to save browser history.

## Contribution

There are many ways you can contribute to Mnemnk:

- Agent development: Create new agents to extend functionality
- Bug reporting: Help identify and fix issues
- Documentation improvement: Help make Mnemnk easier to use
- Feature requests: Share your ideas for new capabilities

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

* API agent
  * [axum](https://github.com/tokio-rs/axum)
  * [tower](https://github.com/tower-rs/tower)

<!----------------------------------------------------------------------------->

[License]: LICENSE

<!----------------------------------{ Badges }--------------------------------->

[Badge Workflow]: https://github.com/mnemnk/mnemnk-app/actions/workflows/publish.yml/badge.svg
[Badge Language]: https://img.shields.io/github/languages/top/mnemnk/mnemnk-app
[Badge License]: https://img.shields.io/github/license/mnemnk/mnemnk-app
