const anchor = require('@project-serum/anchor');

const {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  Token,
  // createMint,
  // getAssociatedTokenAddress,
  // createAssociatedTokenAccountInstruction
} = require("@solana/spl-token");
const { assert } = require('chai');

describe('seer', () => {
  const programName = "seer567890";
  const provider = anchor.Provider.env();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.Seer;

  let usdcToken = null;

  it('Is initialized!', async () => {
    // Add your test here.
     usdcToken = await Token.createMint(
       provider.connection,
       provider.wallet.payer,
       provider.wallet.publicKey,
       null,
       6,
       TOKEN_PROGRAM_ID
     )
    //  console.log(usdcToken)

    let bumps = new PoolBumps();

    const [mainAccount, mainAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(programName)],
      program.programId
    );
    bumps.mainAccount = mainAccountBump;

    const [poolUsdc, poolUsdcBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(programName), Buffer.from("pool_usdc")],
      program.programId
    );
    bumps.poolUsdc = poolUsdcBump;

    const [redeemableMint, redeemableMintBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(programName), Buffer.from("redeemable_mint")],
      program.programId
    );
    bumps.redeemableMint = redeemableMintBump;

    const [userData, userDataBump] = await anchor.web3.PublicKey.findProgramAddress(
      [program.provider.wallet.publicKey.toBuffer()],
      program.programId
    );
    bumps.userData = userDataBump;


    const tx = await program.rpc.initialize(
      programName,
      bumps,
      {
        accounts: {
          userData,
          mainAccount,
          redeemableMint,
          poolUsdc,
          usdcToken: usdcToken.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          user: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY
        }
      }
    );
    console.log("Your transaction signature", tx);
  });

  let userUsdc =null;
  const deposit = new anchor.BN(10_000_000);

  it('transfers usdc from user to pool!', async () => {
    userUsdc = await Token.getAssociatedTokenAddress(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      usdcToken.publicKey,
      program.provider.wallet.publicKey
    )

    let createUserUsdcInstr = Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      usdcToken.publicKey,
      userUsdc,
      program.provider.wallet.publicKey,
      program.provider.wallet.publicKey
    )

    let createUserUsdcTrans = new anchor.web3.Transaction().add(createUserUsdcInstr);
    await provider.send(createUserUsdcTrans);

    await usdcToken.mintTo(
      userUsdc,
      provider.wallet.publicKey,
      [],
      deposit.toString()
    );

    const [mainAccount, mainAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(programName)],
      program.programId
    );

    const [userData, userDataBump] = await anchor.web3.PublicKey.findProgramAddress(
      [program.provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    const [redeemableMint, redeemableMintBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(programName),
        Buffer.from("redeemable_mint")
      ],
      program.programId
    );

    const [userRedeemable, userRedeemableBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        provider.wallet.publicKey.toBuffer(),
        Buffer.from(programName),
        Buffer.from("user_redeemable")
      ],
      program.programId
    );

    const [poolUsdc, poolUsdcBump]  = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(programName),
        Buffer.from("pool_usdc")
      ],
      program.programId
    )

    try {
      const tx = await program.rpc.depositUsdc(deposit,{
        accounts: {
          userData,
          userUsdc,
          userRedeemable,
          mainAccount,
          redeemableMint,
          poolUsdc,
          usdcMint: usdcToken.publicKey,
          user: provider.wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId
        },
        instructions: [
          program.instruction.initUserRedeemable({
            accounts: {
              userRedeemable,
              mainAccount,
              redeemableMint,
              user: provider.wallet.publicKey,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
              rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            }
          })
        ]
      })
    } catch (error) {
      console.log(error.toString());
    }

    poolUsdcBalance = await provider.connection.getTokenAccountBalance(poolUsdc);
    let amountInPool = new anchor.BN(poolUsdcBalance.value.amount);
    assert.ok(amountInPool.eq(deposit));

    userRedeemableBalance = await provider.connection.getTokenAccountBalance(userRedeemable);
    let amountInUser = new anchor.BN(userRedeemableBalance.value.amount);
    assert.ok(amountInUser.eq(deposit));
  });

  it('transfers usdc from pool to user!', async () => {
    const [mainAccount, mainAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(programName)
      ],
      program.programId
    );

    const [redeemableMint, redeemableMintBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(programName),
        Buffer.from("redeemable_mint")
      ],
      program.programId
    );

    const [userRedeemable, userRedeemableBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        provider.wallet.publicKey.toBuffer(),
        Buffer.from(programName),
        Buffer.from("user_redeemable")
      ],
      program.programId
    );

    const [poolUsdc, poolUsdcBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(programName),
        Buffer.from("pool_usdc")
      ],
      program.programId
    );

    const [userEscrow, userEscrowBump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        provider.wallet.publicKey.toBuffer(),
        Buffer.from(programName),
        Buffer.from("user_escrow")
      ],
      program.programId
    )
    
    try {
      const tx = await program.rpc.withdrawUsdc(deposit,{
        accounts: {
          // userData,
          userEscrow,
          userRedeemable,
          mainAccount,
          redeemableMint,
          poolUsdc,
          usdcMint: usdcToken.publicKey,
          user: provider.wallet.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId
        },
        instructions: [
          program.instruction.initUserEscrow({
            accounts: {
              userEscrow,
              mainAccount,
              usdcMint: usdcToken.publicKey,
              user: provider.wallet.publicKey,
              systemProgram: anchor.web3.SystemProgram.programId,
              tokenProgram: TOKEN_PROGRAM_ID,
              rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            }
          })
        ]
      })
    } catch (error) {
      console.log(error.toString());
    }


  });


  function PoolBumps() {
    this.mainAccount;
    this.redeemableMint;
    this.poolUsdc;
    this.userData;
  }
});
