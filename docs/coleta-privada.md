---
type: Guide
title: Handoff privado para coleta de cassettes do SEI
description: Roteiro de gravação delegada, sem publicação automática nem credenciais no Git
---

# Coleta privada em máquina autorizada

O [roteiro](../scripts/coletar-cassette-privada.sh) é uma **ponte para um
coletor externo aprovado**, não um coletor do SEI nem um cliente que saiba
reproduzir sozinho os fluxos Web. Sem `--executar`, limita-se à prévia,
sem rede, sem abrir arquivos ou criar diretórios.

O operador deve obter autorização real para **instalação + identidade +
unidade + operação** antes do comando. `--autorizar-live` é declaração do
operador, não substitui autorização do administrador nem atesta a ausência
de efeitos de GET/POST. O gravador externo é um executável absoluto, sem
symlink, e recebe por variáveis de ambiente somente **caminhos privados**
(`IES_URL_CONTROLE_FILE`, `IES_COOKIE_FILE`, `IES_ORIGEM_FILE`), além de
`IES_CAPTURE_INSTALLATION`, `IES_CAPTURE_OPERATION`, `IES_CAPTURE_SCENARIO`
e `IES_CAPTURE_OUTPUT_DIR`. Não recebe senhas ou cookies como argumentos.

Exemplo de prévia (não executa):

```sh
bash scripts/coletar-cassette-privada.sh \
  --instalacao unb --operacao documento.obter \
  --cenario documento-numero-e-busca-unb \
  --destino /privado/captura-unb
```

Para executar **somente na máquina com autorização**, adicionar:
`--executar --autorizar-live --gravador /privado/coletor-aprovado
--url-file /privado/url --cookie-file /privado/cookie
--origem-file /privado/origem`. Os arquivos devem existir, ser regulares,
não vazios, sem links simbólicos e sem permissões de grupo/outros em Unix.
O destino deve ser absoluto e **novo**. Saída, erros e gravação bruta ficam
em pasta criada com modo privado, com hashes SHA-256 e manifesto local.
Não inserir segredos no nome do cenário, no nome do arquivo nem no diretório.

**O resultado bruto nunca está pronto para publicação.** O manifesto é
local, com estado `coleta_privada_nao_sanitizada`. Depois, outro processo
revisa efeitos, apaga URLs assinadas, cookies e dados pessoais, calcula
novos hashes sobre o material saneado, verifica revisão e só então publica
se autorizado. O script não saneia, não valida semanticamente o HTML e
não envia arquivos ao Internet Archive nem ao GitHub.

Validação multi-instalação de replays existentes continua em
[`validar-cassettes.sh`](../scripts/validar-cassettes.sh); para acesso live,
executar só cenários independentes com permissões e efeitos avaliados.
Teste local sintético: `bash tests/coleta_privada.sh`, sem rede.
