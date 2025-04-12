export const makeTokensProvider = () => {
  return {
    defaultToken: "",
    ignoreCase: true,
    tokenPostfix: ".riscv",
    keywords: [
      ".data",
      ".text",
      ".section",
      ".align",
      ".equ",
      "ADD",
      "SUB",
      "SLT",
      "SLTU",
      "AND",
      "OR",
      "XOR",
      "SLL",
      "SRL",
      "SRA",
      "ADDI",
      "SLTI",
      "SLTIU",
      "ANDI",
      "ORI",
      "XORI",
      "SLLI",
      "SRLI",
      "SRAI",
      "JALR",
      "LW",
      "LH",
      "LHU",
      "LB",
      "LBU",
      "FENCE",
      "ECALL",
      "EBREAK",
      "SW",
      "SH",
      "SB",
      "BEQ",
      "BNE",
      "BLT",
      "BLTU",
      "BGE",
      "BGEU",
      "LUI",
      "AUIPC",
      "JAL"
    ],
    symbols: /[\.,\:]+/,
    escapes: /\\(?:[abfnrtv\\"'$]|x[0-9A-Fa-f]{1,4}|u[0-9A-Fa-f]{4}|U[0-9A-Fa-f]{8})/,
    tokenizer: {
      root: [
        [/x(?:[1-2][0-9]|3[0-1]|[0-9])|pc|zero/, "variable.predefined"],
        [
          /[.a-zA-Z_]\w*/,
          {
            cases: {
              this: "variable.predefined",
              "@keywords": { token: "keyword.$0" },
              "@default": ""
            }
          }
        ],
        [/[ \t\r\n]+/, ""],
        [/#.*$/, "comment"],
        [/@symbols/, "delimiter"],
        [/\d+[eE]([\-+]?\d+)?/, "number.float"],
        [/\d+\.\d+([eE][\-+]?\d+)?/, "number.float"],
        [/0[xX][0-9a-fA-F]+/, "number.hex"],
        [/0[0-7]+(?!\d)/, "number.octal"],
        [/\d+/, "number"],
        [/[,.]/, "delimiter"],
        [/"""/, "string"],
        [/'''/, "string"],
        [
          /"/,
          {
            cases: {
              "@eos": "string",
              "@default": { token: "string", next: '@string."' }
            }
          }
        ],
        [
          /'/,
          {
            cases: {
              "@eos": "string",
              "@default": { token: "string", next: "@string.'" }
            }
          }
        ]
      ],
      string: [
        [/[^"'\#\\]+/, "string"],
        [/@escapes/, "string.escape"],
        [/\./, "string.escape.invalid"],
        [/\./, "string.escape.invalid"],
        [
          /#{/,
          {
            cases: {
              '$S2=="': {
                token: "string",
                next: "root.interpolatedstring"
              },
              "@default": "string"
            }
          }
        ],
        [
          /["']/,
          {
            cases: {
              "$#==$S2": { token: "string", next: "@pop" },
              "@default": "string"
            }
          }
        ],
        [/#/, "string"]
      ],
    }
  }
};
