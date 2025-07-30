# PurrMint 🔨

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70+-blue.svg)](https://www.rust-lang.org/)
[![Android](https://img.shields.io/badge/Android-API%2021+-green.svg)](https://developer.android.com/)

**Mobile Cashu Mint** - Run your own Cashu mint directly on your Android phone! Built with Rust + Kotlin for maximum security and performance.

## What is PurrMint?

PurrMint is a mobile Cashu mint service that transforms your Android device into a personal ecash mint. Whether you want a private mint for personal use or a public mint for others, PurrMint makes it possible with just a few taps.

## Core Features

### **Mode Support**
- **Local Mint Mode**: Run a local mint service accessible only from your device for maximum privacy
- **Tor Mint Mode**: Generate onion addresses allowing external access through the Tor network for public mint operations

### **Lightning Configuration Support**
- **Fake Wallet**: Perfect for testing and development
- **LNbits**: Lightning Network bits integration
- **CLN**: Core Lightning support
- **NWC (Nostr Wallet Connect)**: Connect to any Lightning wallet that supports NWC protocol

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
   - Use **NWC** to connect to any compatible Lightning wallet

### 3. Connect Your Wallet

1. **Open wallet.cashu.me** in your mobile browser
2. **Add your mint**: `127.0.0.1:3338`
3. **Start using ecash**:
   - Receive Lightning payments
   - Send ecash to others
   - Manage your tokens

## Lightning Backend Configuration

### NWC (Nostr Wallet Connect)

NWC allows you to connect PurrMint to any Lightning wallet that supports the Nostr Wallet Connect protocol. This includes popular wallets like:

- **Alby**
- **Zeus**
- **Phoenix**
- **Blixt**
- **Mutiny**
- And many others

#### Setting up NWC:

1. **Get your NWC connection URI** from your Lightning wallet
   - Usually found in wallet settings under "Connect" or "Nostr Wallet Connect"
   - Format: `nostr+walletconnect://...`

2. **Configure PurrMint**:
   - Select "NWC" as your Lightning backend
   - Paste your NWC connection URI
   - The mint will automatically connect to your wallet

## Roadmap

### Coming Soon
- **Enhanced Tor Integration**: Improved onion address generation and management
- **Advanced UI**: Better user experience and configuration options
- **Multi-mint Management**: Run multiple mints from one device

### Future Plans
- **Advanced Analytics**: Mint usage statistics and insights
- **Plugin System**: Extensible architecture for custom features
- **Cross-platform Support**: iOS and desktop versions

## Documentation

- [Cashu Protocol](https://github.com/cashubtc/cashu) - Understanding Cashu
- [CDK Documentation](https://github.com/cashubtc/cdk) - Cashu Development Kit
- [Arti](https://gitlab.torproject.org/tpo/core/arti) - Tor implementation in Rust
- [NIP-47](https://github.com/nostr-protocol/nips/blob/master/47.md) - Nostr Wallet Connect specification

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.