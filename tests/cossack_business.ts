import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ResourceManager } from "../target/types/resource_manager";
import { ItemNft } from "../target/types/item_nft";
import { MagicToken } from "../target/types/magic_token";
import { Marketplace } from "../target/types/marketplace";
import { Search } from "../target/types/search";
import {
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountInstruction,
  createInitializeMint2Instruction,
  getMintLen,
} from "@solana/spl-token";
import { expect } from "chai";

describe("cossack_business_multi_resource", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const resourceProgram = anchor.workspace.ResourceManager as Program<ResourceManager>;
  const itemNftProgram = anchor.workspace.ItemNft as Program<ItemNft>;
  const searchProgram = anchor.workspace.Search as Program<Search>;
  const magicTokenProgram = anchor.workspace.MagicToken as Program<MagicToken>;
  const marketplaceProgram = anchor.workspace.Marketplace as Program<Marketplace>;

  const admin = provider.wallet;

  let woodMint = anchor.web3.Keypair.generate();
  let ironMint = anchor.web3.Keypair.generate();
  let leatherMint = anchor.web3.Keypair.generate();
  
  let itemMintKeypair = anchor.web3.Keypair.generate();
  let magicTokenMint = anchor.web3.Keypair.generate();
  
  let playerWoodATA: anchor.web3.PublicKey;
  let playerIronATA: anchor.web3.PublicKey;
  let playerLeatherATA: anchor.web3.PublicKey;
  let playerItemATA: anchor.web3.PublicKey;
  let playerMagicATA: anchor.web3.PublicKey;

  const [gameConfigPDA] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("game_config")], resourceProgram.programId);
  const [nftAuthorityPDA] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("nft_authority")], itemNftProgram.programId);
  const [magicMintAuthority] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("mint_authority")], magicTokenProgram.programId);
  const [playerPDA] = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("player"), admin.publicKey.toBuffer()], searchProgram.programId);

  it("1. Ініціалізація інфраструктури", async () => {
    const configExist = await provider.connection.getAccountInfo(gameConfigPDA);
    if (!configExist) {
      await resourceProgram.methods.initializeGame().accounts({
        admin: admin.publicKey,
        gameConfig: gameConfigPDA,
        systemProgram: anchor.web3.SystemProgram.programId,
      }).rpc();
    }

    const lamports = await provider.connection.getMinimumBalanceForRentExemption(getMintLen([]));
    const tx = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: admin.publicKey,
        newAccountPubkey: magicTokenMint.publicKey,
        space: getMintLen([]),
        lamports,
        programId: TOKEN_2022_PROGRAM_ID,
      }),
      createInitializeMint2Instruction(magicTokenMint.publicKey, 0, magicMintAuthority, null, TOKEN_2022_PROGRAM_ID)
    );
    await provider.sendAndConfirm(tx, [magicTokenMint]);
    playerMagicATA = getAssociatedTokenAddressSync(magicTokenMint.publicKey, admin.publicKey, false, TOKEN_2022_PROGRAM_ID);
  });

  it("2. Створення мінтів для всіх ресурсів", async () => {
    await resourceProgram.methods.createResourceMint(0).accounts({
        admin: admin.publicKey,
        gameConfig: gameConfigPDA,
        resourceMint: woodMint.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
    }).signers([woodMint]).rpc();

    await resourceProgram.methods.createResourceMint(1).accounts({
        admin: admin.publicKey,
        gameConfig: gameConfigPDA,
        resourceMint: ironMint.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
    }).signers([ironMint]).rpc();

    await resourceProgram.methods.createResourceMint(3).accounts({
        admin: admin.publicKey,
        gameConfig: gameConfigPDA,
        resourceMint: leatherMint.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
    }).signers([leatherMint]).rpc();

    playerWoodATA = getAssociatedTokenAddressSync(woodMint.publicKey, admin.publicKey, false, TOKEN_2022_PROGRAM_ID);
    playerIronATA = getAssociatedTokenAddressSync(ironMint.publicKey, admin.publicKey, false, TOKEN_2022_PROGRAM_ID);
    playerLeatherATA = getAssociatedTokenAddressSync(leatherMint.publicKey, admin.publicKey, false, TOKEN_2022_PROGRAM_ID);

    const tx = new anchor.web3.Transaction().add(
        createAssociatedTokenAccountInstruction(admin.publicKey, playerWoodATA, admin.publicKey, woodMint.publicKey, TOKEN_2022_PROGRAM_ID),
        createAssociatedTokenAccountInstruction(admin.publicKey, playerIronATA, admin.publicKey, ironMint.publicKey, TOKEN_2022_PROGRAM_ID),
        createAssociatedTokenAccountInstruction(admin.publicKey, playerLeatherATA, admin.publicKey, leatherMint.publicKey, TOKEN_2022_PROGRAM_ID)
    );
    await provider.sendAndConfirm(tx);
  });

  it("3. Поповнення ресурсів для крафту Шаблі", async () => {
    await resourceProgram.methods.mintResource(0, new anchor.BN(10)).accounts({
        gameConfig: gameConfigPDA,
        resourceMint: woodMint.publicKey,
        playerTokenAccount: playerWoodATA,
        player: admin.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
    }).rpc();

    await resourceProgram.methods.mintResource(1, new anchor.BN(10)).accounts({
        gameConfig: gameConfigPDA,
        resourceMint: ironMint.publicKey,
        playerTokenAccount: playerIronATA,
        player: admin.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
    }).rpc();

    await resourceProgram.methods.mintResource(3, new anchor.BN(10)).accounts({
        gameConfig: gameConfigPDA,
        resourceMint: leatherMint.publicKey,
        playerTokenAccount: playerLeatherATA,
        player: admin.publicKey,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
    }).rpc();

    console.log("Склад ресурсів поповнено: Дерево, Залізо, Шкіра.");
  });

  it("4. Крафт Шаблі (ID 1) з багатьох ресурсів", async () => {
    playerItemATA = getAssociatedTokenAddressSync(itemMintKeypair.publicKey, admin.publicKey, false, TOKEN_2022_PROGRAM_ID);
    
    const mintLen = getMintLen([]);
    const lamports = await provider.connection.getMinimumBalanceForRentExemption(mintLen);
    const tx = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: admin.publicKey,
        newAccountPubkey: itemMintKeypair.publicKey,
        space: mintLen,
        lamports,
        programId: TOKEN_2022_PROGRAM_ID,
      }),
      createInitializeMint2Instruction(itemMintKeypair.publicKey, 0, nftAuthorityPDA, null, TOKEN_2022_PROGRAM_ID),
      createAssociatedTokenAccountInstruction(admin.publicKey, playerItemATA, admin.publicKey, itemMintKeypair.publicKey, TOKEN_2022_PROGRAM_ID)
    );
    await provider.sendAndConfirm(tx, [itemMintKeypair]);

    await itemNftProgram.methods.craftItem(1).accounts({
        player: admin.publicKey,
        gameConfig: gameConfigPDA,
        woodMint: woodMint.publicKey,
        playerWoodAccount: playerWoodATA,
        ironMint: ironMint.publicKey,
        playerIronAccount: playerIronATA,
        leatherMint: leatherMint.publicKey,
        playerLeatherAccount: playerLeatherATA,
        itemMint: itemMintKeypair.publicKey,
        playerItemAccount: playerItemATA,
        nftAuthority: nftAuthorityPDA,
        resourceManagerProgram: resourceProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
    }).rpc();

    const woodBal = await provider.connection.getTokenAccountBalance(playerWoodATA);
    console.log("Залишок дерева:", woodBal.value.uiAmount);
    expect(Number(woodBal.value.uiAmount)).to.equal(9);
  });

  it("5. Продаж Шаблі на Маркетплейсі", async () => {
    const tx = new anchor.web3.Transaction().add(
      createAssociatedTokenAccountInstruction(admin.publicKey, playerMagicATA, admin.publicKey, magicTokenMint.publicKey, TOKEN_2022_PROGRAM_ID)
    );
    await provider.sendAndConfirm(tx);

    await marketplaceProgram.methods.sellItem(1).accounts({
        player: admin.publicKey,
        gameConfig: gameConfigPDA,
        nftMint: itemMintKeypair.publicKey,
        playerNftAccount: playerItemATA,
        magicTokenMint: magicTokenMint.publicKey,
        playerMagicTokenAccount: playerMagicATA,
        magicTokenAuthority: magicMintAuthority,
        magicTokenProgram: magicTokenProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
    }).rpc();

    const bal = await provider.connection.getTokenAccountBalance(playerMagicATA);
    console.log("Прибуток у MagicTokens:", bal.value.uiAmount);
    expect(Number(bal.value.uiAmount)).to.be.greaterThan(0);
  });
});