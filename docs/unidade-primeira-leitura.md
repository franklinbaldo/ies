---
type: Guide
title: Primeira leitura da caixa do SEI
description: GET seguro de uma caixa já autenticada e replay das cassettes pelos testes do acervo
---

# Primeira leitura de processos da unidade

O IES Rust interpreta duas tabelas independentes da caixa SEI:
`tblProcessosRecebidos` e `tblProcessosGerados`. O resultado JSON
contém número, identificador interno, visualização e total declarado.
O valor `completa: bool` é calculado na borda a partir da cobertura
conferida, jamais armazenado no domínio.

**O GET ao vivo continua lendo somente a página atual.**
Não inclui troca de unidade, POST ou escrita no SEI; o login é o comando
à parte [`ies login`](../contracts/cli/sessao-login.md).

## Replay das cassettes do Internet Archive

Não há `ies replay` (retirado em 05/11/2026,
[decisão](decisoes/acervo/replay-fora-da-cli.md)). As respostas gravadas do
Controle passam pelos mesmos leitores da caixa no teste ignorado
`visao_do_controle::acervo::replay_do_controle_inteiro`, que confere SHA-1
do ZIP, SHA-256 normalizado da cassette, host permitido, schema e status,
e não publica cookies, HTML bruto nem URLs `infra_hash`:

```sh
scripts/replay visao_do_controle::acervo
```

Os ZIPs ficam em `~/.cache/ies` (ou `IES_ACERVO_CACHE`); como baixá-los e
conferi-los está em [cassettes](cassettes.md#replay-do-acervo).

## Prévia segura dos arquivos de sessão, sem rede

Antes de usar `ies unidade listar-processos`, é possível testar
localmente o formato da URL assinada, o caminho-base HTTPS da instalação
e a sintaxe do Cookie, sem enviar nenhuma requisição:

```sh
IES_URL_CONTROLE_FILE=/caminho/privado/url-caixa \
IES_COOKIE_FILE=/caminho/privado/cookie \
IES_ORIGEM_FILE=/caminho/privado/origem \
ies status --verificar-sessao-local --json
```

A verificação compartilha com o transporte as regras de arquivo privado
regular, permissões `0600`, assinatura de `procedimento_controlar`,
origem e cabeçalho Cookie. Valida a origem antes de abrir o cookie.
O resultado positivo informa `sessao_local: formato_e_origem_verificados`,
mas **não** prova que o SEI esteja acessível, autenticado ou que o
`infra_hash` continue válido: `conexao: nao_verificada` e
`acesso_remoto: false` permanecem explícitos. Nenhum valor confidencial
é mostrado. O modo `--verificar-arquivos` antigo lê somente metadados;
não confundir as duas verificações.

## Leitura HTTPS de caixa assinada

Para uma sessão **já autenticada**, o IES aceita exclusivamente
GET de URL HTTPS publicada pela própria caixa do SEI, assinada na
mesma sessão **e no host da origem explicitamente autorizada**.
Forneça três arquivos privados `0600`: URL assinada, cookie e
origem, contendo o endereço-base HTTPS da instalação
(`https://host` ou `https://host/caminho`). O arquivo admite
opcionalmente uma segunda linha, `versao=5.0.4`, apenas como
metadado declarado. Host **e caminho-base** são conferidos antes
de enviar cookies; outra instalação no mesmo host é recusada.
O cookie contém apenas os pares da linha `Cookie`
(por exemplo, `PHPSESSID=...; OUTRO=...`), sem prefixo `Cookie:`.
O endereço-base é conferido antes de ler os cookies ou fazer rede.
A versão não altera permissões nem habilita recursos.

```sh
chmod 600 /caminho/privado/cookies /caminho/privado/url-caixa /caminho/privado/origem
IES_URL_CONTROLE_FILE=/caminho/privado/url-caixa \\
IES_COOKIE_FILE=/caminho/privado/cookies \\
IES_ORIGEM_FILE=/caminho/privado/origem \\
ies unidade listar-processos --json
```

O exemplo de URL é ilustrativo: **não construa nem adivinhe o hash**.
Obtenha-o intacto no navegador ou cliente autorizado. O programa
não segue redirects, não cria login (isso é o `ies login`) e não repete pedidos
automaticamente; só `ies unidade recebidos|gerados --pagina N` faz POST. Falhas não imprimem cookies ou URL assinada.

O `lnkInfraUnidade` no HTML é o rótulo exibido pela sessão,
não um ID interno nem uma URL de navegação. `--unidade` agora
verifica esse rótulo como pré-condição, sem fabricá-lo. Quando
o marcador não está presente, o JSON apresenta `unidade: null`
e não aceita uma asserção de unidade sem verificação.

Falhas distinguem origem não autorizada, resposta excessiva,
acesso negado, redirect e integridade da cassette por subcódigos
JSON, mas não exibem cookies ou respostas autenticadas.

O `lnkInfraUnidade` no HTML é o rótulo exibido pela sessão,
não um ID interno nem uma URL de navegação. `--unidade` agora
verifica esse rótulo como pré-condição, sem fabricá-lo. Quando
o marcador não está presente, o JSON apresenta `unidade: null`
e não aceita uma asserção de unidade sem verificação.

Falhas distinguem origem não autorizada, resposta excessiva,
acesso negado, redirect e integridade da cassette por subcódigos
JSON, mas não exibem cookies ou respostas autenticadas.

Em arquivos `--saida`, a escrita usa modo `0600` em Unix,
recusa sobrescrever outro arquivo e produz apenas o resultado
estruturado. Os dados da caixa podem ser sensíveis: não envie
essas saídas ao Git ou ao Internet Archive.

## Contrato JSON v2 simplificado

O domínio guarda `IdProcedimento`, `Processo`, `Tabela<Processo>` e
`CaixaDeEntrada`, sem enum de completude nem campos de apresentação.
A borda em `src/saida.rs` calcula o booleano de cobertura e formata
JSON ou saída humana, sem misturar metadados de origem com parsing HTML.

O JSON é o array `[recebidos, gerados]`; cada lista tem `unidade`,
`origem`, `categoria`, `total_declarado`, `faixa_observada` e
`processos` ([forma única](../contracts/cli/unidade-recebidos-gerados.md));
cada processo tem `id`, `numero`,
`visualizado` e `sigiloso`, lidos da classe do link. A prova de
cobertura não é exposta como estado mutável. O
[golden de saída](../tests/fixtures/caixa-v2.json)
fixa esse contrato byte a byte num teste unitário
(`src/saida/tests_golden.rs`).

## Testes e limites

- UnB: 25 recebidos e 18 gerados; ambas as tabelas completas.
- PEN3: 100 recebidos de 168 e 91 gerados de 91; parcial.
- Goiás: 10 recebidos de 131 e 10 gerados de 7939; parcial.

Os três resultados foram reproduzidos offline (pelo antigo `ies replay`) a partir dos ZIPs
públicos, sem gravações no repositório. O `cargo test` utiliza
HTML sintético para proteger parser e transportes contra regressões.

Atualização (#57): a caixa paginada passa a ser devolvida como página
observada com total e faixa; `ies unidade recebidos|gerados --pagina N`
reconstrói um POST do formulário servido, sem restaurar a página da sessão
([páginas da caixa](../contracts/cli/unidade-listar-processos-paginas.md)).
A saída é a [forma única de lista](../contracts/cli/unidade-recebidos-gerados.md):
`listar-processos` devolve o array `[recebidos, gerados]`, e os
subcomandos, uma lista. O ensaio live de 02/11/2026 usou a sintaxe
anterior `listar-processos --categoria T --pagina N`.
