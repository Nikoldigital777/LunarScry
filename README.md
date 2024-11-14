# LunarScry

**LunarScry** is an AI-powered decentralized content moderation protocol built on the Solana blockchain. It leverages **Groq's LLaMA 3.2 model** for rapid content analysis of images, text, links, and potential scams, and combines this with **community governance** through stake-weighted voting to ensure fair and transparent moderation decisions.

## Purpose

LunarScry allows decentralized applications (dApps) to automate the detection of harmful content (e.g., harassment, spam, scams, or inappropriate images and links) using AI. The community can then vote on whether flagged content should be approved or rejected. This hybrid approach ensures that content moderation is both efficient and fair, with economic incentives for participants.

## Key Features

- **AI-Powered Moderation**: Automatically detects harmful content such as spam, hate speech, scams, inappropriate images, and malicious links using Groq's LLaMA 3.2.
- **Decentralized Voting**: Community members vote on flagged content using a stake-weighted system, ensuring fairness and transparency.
- **Token Economics**: The GUARD token is used for staking, voting, and distributing rewards to participants.
- **Developer-Friendly SDK**: Easy integration with dApps through a TypeScript SDK that allows interaction with LunarScry's smart contracts.

## How It Works

1. **Content Submission**: Users submit content (text, images, or links) for moderation. This content is analyzed by Groq's LLaMA 3.2 AI model for harmful elements like harassment, scams, or inappropriate imagery.
2. **AI Analysis**: The AI assigns a confidence score based on its analysis of the content. If the score is above a certain threshold (e.g., 50%), the content is flagged for community review.
3. **Community Voting**: If flagged by the AI, the community votes on whether to approve or reject the content. Voting is stake-weighted, meaning users with more tokens have greater influence.
4. **Final Decision**: Once voting concludes, if quorum is reached (a minimum number of votes), the decision is finalized as either "approved" or "rejected."
5. **Rewards Distribution**: Users who voted in line with the final decision are rewarded with tokens based on their stake and voting time.

## Getting Started

### Prerequisites

Ensure you have the following tools installed:
- [Rust](https://www.rust-lang.org/) (for smart contract development)
- [Anchor](https://project-serum.github.io/anchor/getting-started/introduction.html) (Solana framework)
- [Node.js](https://nodejs.org/) (for frontend and SDK development)

### Installation

1. Clone this repository:

```bash
git clone https://github.com/YOUR_GITHUB_USERNAME/LunarScry.git
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

## Folder Structure

```bash
LunarScry/
├── contracts/          # Smart contracts written in Rust (Anchor framework)
│   ├── Cargo.toml      # Rust project file
│   ├── src/
│   │   ├── lib.rs      # Main contract logic
│   │   ├── errors.rs   # Error handling
│   │   ├── events.rs   # Event definitions
│   │   └── state.rs    # Account state definitions
├── sdk/                # SDK for dApp integration (TypeScript)
│   ├── package.json    # NPM package file
│   ├── index.ts        # Main SDK file for interacting with LunarScry contracts
├── frontend/           # Frontend code (React.js)
│   ├── package.json    # NPM package file for frontend dependencies
│   ├── src/
│   │   ├── App.tsx     # Main React app logic
├── docs/               # Documentation files
│   ├── README.md       # Main README file for the repository
│   ├── API.md          # API documentation for SDK and contracts
└── LICENSE             # Open-source license (MIT or Apache 2.0)
```

## API Documentation

### Installation

Install the SDK via npm:

```bash
npm install @lunarscry/sdk --save
```

### Example Usage

#### Submit Content

```typescript
import { LunarScry } from '@lunarscry/sdk';

const lunarscry = new LunarScry({
  apiKey: 'your-api-key',
  network: 'devnet', // or 'mainnet'
});

async function moderateContent(content: string) {
  const result = await lunarscry.analyze({
    content,
    contentType: 'text', // Can also be 'image' or 'link'
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

For more detailed examples, refer to the full [API Documentation](docs/API.md).

## Contributing

We welcome contributions from developers of all skill levels! Here's how you can get involved:

1. Fork the repository.
2. Create a new branch (`git checkout -b feature/my-feature`).
3. Make your changes.
4. Commit your changes (`git commit -m 'Add new feature'`).
5. Push your branch (`git push origin feature/my-feature`).
6. Open a pull request.

Please follow these guidelines when writing code:
- Use Rust's standard formatting (`cargo fmt`) for smart contracts.
- Use ESLint rules provided in the frontend and SDK projects.
- Write descriptive commit messages.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
