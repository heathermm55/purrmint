# PurrMint 🔨

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70+-blue.svg)](https://www.rust-lang.org/)
[![Android](https://img.shields.io/badge/Android-API%2021+-green.svg)](https://developer.android.com/)

**Mobile Cashu Mint** - Run your own Cashu mint directly on your Android phone! Built with Rust + Kotlin for maximum security and performance.

## What is PurrMint?

PurrMint is a mobile Cashu mint service that transforms your Android device into a personal ecash mint. Whether you want a private mint for personal use or a public mint for others, PurrMint makes it possible with just a few taps.

## Core Features

### **Multi-Mode Operation**
- **Mintd HTTP Mode**: Start a local HTTP service for private mint operations
- **NIP-74 Mode**: Run as a public mint using Nostr protocol for decentralized communication (coming soon)
- **Onion Mode**: Generate onion addresses for your mint (coming soon)

### **Lightning Configuration Support**
- **Fake Wallet**: Perfect for testing and development
- **LNbits**: Lightning Network bits integration
- **CLN**: Core Lightning support

## 📱 Quick Start

### 1. Build & Install

```bash
# Clone and build
git clone https://github.com/purrmint/purrmint.git
cd purrmint
./build-release.sh

# Install the APK on your Android device
```

### 2. How to test Your Mint

1. **Open the PurrMint app**
2. **Login with your Nostr account**
3. **Select Lightning backend**:
   - Start with **Fake Wallet** for testing
   - Configure **LNbits/CLN** for real Lightning

### 3. Connect Your Wallet

1. **Open wallet.cashu.me** in your mobile browser
2. **Add your mint**: `127.0.0.1:3338`
3. **Start using ecash**:
   - Receive Lightning payments
   - Send ecash to others
   - Manage your tokens

## Roadmap

### Coming Soon
- **Tor Integration**: Native Tor support for automatic onion address generation
- **Enhanced NIP-74**: Better compatibility with more Cashu wallets
- **Advanced UI**: Improved user experience and configuration options

### Future Plans
- **Multi-mint Management**: Run multiple mints from one device
- **Advanced Analytics**: Mint usage statistics and insights
- **Plugin System**: Extensible architecture for custom features

## Documentation

- [NIP-74 Specification](https://github.com/heathermm55/nips/blob/master/74.md) - The protocol that powers public mints
- [Cashu Protocol](https://github.com/cashubtc/cashu) - Understanding Cashu
- [CDK Documentation](https://github.com/cashubtc/cdk) - Cashu Development Kit

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Transform your Android device into a Cashu mint with PurrMint! 🔨⚡**