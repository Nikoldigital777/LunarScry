# LunarScry

**LunarScry** is an AI-powered decentralized content moderation protocol built on Solana. It leverages Groq's LLaMA model for rapid content analysis and combines it with community governance through a stake-weighted voting system.

## Key Features

- **AI-Powered Moderation**: Automated detection of spam, hate speech, scams, and more.
- **Decentralized Governance**: Community-driven voting mechanism for content moderation.
- **Token Economics**: GUARD tokens are used for staking, rewards, and governance.
- **Developer-Friendly SDK**: Easy integration with dApps through our TypeScript SDK.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) (for smart contract development)
- [Anchor](https://project-serum.github.io/anchor/getting-started/introduction.html) (Solana framework)
- [Node.js](https://nodejs.org/) (for frontend and SDK development)

### Installation

1. Clone this repository:

```bash
git clone https://github.com/YOUR_USERNAME/LunarScry.git
cd LunarScry
```

2. Install dependencies:

```bash
# For contracts:
cd contracts/
cargo build-bpf

# For frontend:
cd ../frontend/
npm install

# For SDK:
cd ../sdk/
npm install
```

3. Deploy contracts to Solana devnet:

```bash
anchor deploy --provider.cluster devnet
```

## Documentation

- [API Documentation](docs/API.md): Learn how to interact with LunarScry via our SDK.
- [Contributing Guidelines](docs/CONTRIBUTING.md): How to contribute to LunarScry.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
```

## Overview

LunarScry provides a set of APIs to interact with its decentralized content moderation protocol. The APIs allow you to submit content, cast votes, claim rewards, and more.

### SDK Installation

Install the SDK via npm:

```bash
npm install @lunarscry/sdk --save
```

### Example Usage

#### Submit Content

```typescript
import { LightGuard } from '@lunarscry/sdk';

const lightguard = new LightGuard({
  apiKey: 'your-api-key',
  network: 'devnet', // or 'mainnet'
});

async function moderateContent(content: string) {
  const result = await lightguard.analyze({
    content,
    contentType: 'text',
    callback_url: 'https://your-app.com/callback'
  });

  return result;
}
```

#### Cast Vote

```typescript
import { castVote } from '@lunarscry/sdk';

await castVote({
  contentId: 'CONTENT_ID',
  voteType: 'Approve',
  stakeAmount: 1000,
});
```

For more detailed examples, refer to the full [SDK documentation](https://lunarscry.dev/docs).
```

## 4. **Contributing Guidelines (`docs/CONTRIBUTING.md`)**

Encourage contributions by providing clear guidelines.

```markdown
# Contributing to LunarScry

We welcome contributions from developers of all skill levels! Here's how you can get involved:

## How to Contribute

1. Fork the repository.
2. Create a new branch (`git checkout -b feature/my-feature`).
3. Make your changes.
4. Commit your changes (`git commit -m 'Add new feature'`).
5. Push your branch (`git push origin feature/my-feature`).
6. Open a pull request.

## Code Style

Please follow these guidelines when writing code:

- Use Rust's standard formatting (`cargo fmt`) for smart contracts.
- Use ESLint rules provided in the frontend and SDK projects.
- Write descriptive commit messages.

## Reporting Issues

If you find a bug or have a feature request, please open an issue on GitHub with as much detail as possible.
```

## 5. **License**

Choose an open-source license such as MIT or Apache 2.0 to allow others to use and contribute to your project freely.

```plaintext
MIT License

Copyright (c) 2024 LunarScry

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction...
```
