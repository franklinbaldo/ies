# language: pt
Funcionalidade: Ajuda da CLI IES

  Cenário: Consultar ajuda geral
    Dado que a CLI ies está disponível
    Quando executo ies --help
    Então o código de saída deve ser 0
    E a saída deve apresentar o comando processo
