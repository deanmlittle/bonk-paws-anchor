type SwapQuote = {
    inputMint: string;
    inAmount: string;
    outputMint: string;
    outAmount: string;
    otherAmountThreshold: string;
    swapMode: string;
    slippageBps: number;
    platformFee: null | any; // Use 'any' for unknown structure
    priceImpactPct: string;
    routePlan: SwapPlan;
    contextSlot: number;
    timeTaken: number;
  };

  type SwapInfo = {
    ammKey: string;
    label: string;
    inputMint: string;
    outputMint: string;
    inAmount: string;
    outAmount: string;
    feeAmount: string;
    feeMint: string;
  };
  
  type SwapPlan = {
    swapInfo: SwapInfo;
    percent: number;
  };  
  
  type SwapInstruction = {
    tokenLedgerInstruction: null | any; // Use 'any' for unknown structure
    computeBudgetInstructions: {
      programId: string;
      accounts: any[]; // Use 'any' for unknown structure
      data: string;
    }[];
    setupInstructions: {
      programId: string;
      accounts: any[]; // Use 'any' for unknown structure
      data: string;
    }[];
    swapInstruction: {
      programId: string;
      accounts: any[][]; // Use 'any' for unknown structure
      data: string;
    };
    cleanupInstruction: {
      programId: string;
      accounts: any[]; // Use 'any' for unknown structure
      data: string;
    };
    addressLookupTableAddresses: string[];
  };
  