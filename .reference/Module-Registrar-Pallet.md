# Architecture Design Document: Module Registrar Pallet

## Overview

This document outlines the architecture for the new **Module Registrar Pallet**, designed to replace the subnet system on the Commune blockchain. The Module Registrar will act as a decentralized package manager, enabling the registration, management, and interaction with off-chain modules that provide services such as inference, storage, and validation. This system represents a critical step toward simplifying the Commune blockchain's infrastructure while expanding its capabilities to support a broader range of tools and services.

### Key Features

- **On-Chain Storage:** Minimal metadata, including IPFS hashes of module containers and scripts.
- **Module Validation:** Validators verify modules by building and testing their containers, storing validated artifacts on IPFS.
- **Cross-Module Communication:** Standardized APIs for interaction and runtime RPC for relaying requests.
- **Stake-Based Registration:** Modules and miners require stakes to participate, with slashing for malicious behavior.
- **Trust Scoring:** Adapted from the chain’s existing weight-based system.

---

## Partnership Context

### Commune-AI Module Registrar System

With our partnership with Commune in place, we aim to build a **Module Registrar System** directly on-chain. This system will align with the tooling that **Syn-AI** provides, enabling these tools to be registered as modules on the Commune-AI blockchain. The registrar will replace the previous subnet architecture, which was deemed too complex and limited for the evolving needs of the network.

The Module Registrar will serve as the backbone for integrating Syn-AI's advanced inference tools and memory systems into Commune-AI, allowing seamless interactions between AI services and the blockchain.

### Strategic Goals

- **First Use Case:** Commune-AI will act as the inaugural customer for Syn-AI's memory and state management systems, providing a live testing environment and valuable feedback.
- **Funding Development:** Revenue generated from Commune-AI will fund further development of Syn-AI tools.
- **Bridge Creation:** The funds will also establish a liquidity pool to underwrite a bridge between Commune-AI and Syn-AI, enhancing interoperability and scalability across chains.

---

## High-Level Architecture

### What is the Module Registrar?

The Module Registrar is a system for registering, managing, and interacting with off-chain tools (modules) on the blockchain. These modules act as independent services that users and other modules can query and interact with through standardized APIs.

#### Key Roles:

1. **Modules**: Provide services such as AI inference, data storage, or validation. Developers register these on the blockchain with metadata and build scripts.
2. **Validators**: Ensure modules are functional by verifying their build and testing their behavior. They validate modules before they are made available to the network.
3. **Miners**: Execute the computational work or service requests for modules, earning rewards based on their performance and trustworthiness.
4. **Users**: Query and interact with modules for specific tasks or services.

---

## System Components

### 1. **Module Registrar Pallet**

#### **Core Responsibilities**

- Act as the on-chain registry for modules.
- Store metadata, including IPFS hashes of module containers and scripts.
- Provide CRUD operations for managing modules.
- Facilitate communication between modules, users, and miners.

#### **How It Works**

1. **Module Registration**:

   - Developers submit module metadata, scripts, and a stake to the blockchain.
   - Metadata includes:
     - Module name
     - Version information
     - Repository URL
     - Build and installer script URLs
     - IPFS hash of the validated container

2. **Validation**:

   - Validators build the module's container using the provided script.
   - The container is tested for functionality and compliance with chain standards.
   - Once validated, the container is uploaded to IPFS, and its hash is stored on-chain.

3. **Interaction**:

   - Users query modules through standardized APIs.
   - Miners process requests, ensuring services are delivered efficiently.

#### **Storage Design**

```rust
#[derive(Encode, Decode)]
pub struct ModuleMetadata {
    pub name: BoundedVec<u8, MaxNameLen>,
    pub version_major: u16,
    pub version_minor: u16,
    pub version_patch: u16,
    pub repo_url: BoundedVec<u8, MaxUrlLen>,
    pub build_script_url: BoundedVec<u8, MaxUrlLen>,
    pub installer_script_url: BoundedVec<u8, MaxUrlLen>,
    pub ipfs_hash: H256,
    pub trust_score: u32,
    pub stake: Balance,
}

pub enum ModuleState {
    Registered,
    Active,
    Deprecated,
    Suspended,
}
```

---

### 2. **Validator Role in Module Validation**

Validators are central to ensuring the integrity of the module ecosystem. Their workflow includes:

1. **Submission**: Validators receive metadata and build scripts from developers.
2. **Validation**:
   - Clone the repository.
   - Build the module in a sandboxed environment (e.g., Docker container).
   - Verify functionality using a standardized test suite.
3. **Approval**: Push the validated container to IPFS and record its hash on-chain.
4. **Incentives**: Validators earn emissions for successful validations and face slashing for misconduct.

---

## Off-Chain Components

### 1. **Build and Installer Scripts**

Developers are required to provide scripts for building and installing their modules. These scripts standardize the process and ensure compatibility across environments.

#### Example Build Script

```bash
#!/bin/bash
REPO_URL=$1
MODULE_NAME=$2
git clone $REPO_URL $MODULE_NAME
cd $MODULE_NAME
docker build -t commune-module-$MODULE_NAME .
```

#### Example Installer Script

```bash
#!/bin/bash
BUILD_SCRIPT_URL=$1
MODULE_ID=$2
wget $BUILD_SCRIPT_URL -O build_script.sh
chmod +x build_script.sh
./build_script.sh "https://github.com/org/repo.git" "$MODULE_ID"
```

---

### 2. **Module API Standard**

To ensure interoperability, all modules must implement the following API endpoints:

- `GET /commands`: Lists available commands.
- `POST /execute`: Executes a specified command.
- `GET /state`: Provides the current state of the module.

Example Swagger Spec:

```json
{
  "endpoints": {
    "/commands": {"method": "GET"},
    "/execute": {"method": "POST"},
    "/state": {"method": "GET"}
  }
}
```

---

## Governance and Trust

### 1. **Trust Scoring System**

- Weighted scores assigned by root validators.
- Metrics: Validator feedback, module performance, user reviews.

### 2. **Stake-Based Model**

- **Modules:** Stake is required to register, and slashing occurs for malicious behavior.
- **Miners:** Stake is required to process module requests and earn emissions.

### 3. **DAO Oversight**

- The DAO defines slashing triggers and resolves disputes.
- Clear documentation ensures transparency and fairness.

---

## Summary

The Module Registrar Pallet represents a significant step forward for the Commune blockchain. By replacing the subnet system with a modular, validator-driven framework, it creates a more flexible and secure infrastructure that benefits developers, users, and validators alike. This system also strengthens the partnership between Commune-AI and Syn-AI, driving innovation and cross-chain collaboration.
