---
type: Guide
title: Primeira leitura da caixa do SEI
description: Replay com cassettes verificadas e GET seguro de uma caixa já autenticada
---

# Primeira leitura de processos da unidade

O IES Rust interpreta duas tabelas independentes da caixa SEI:
`tblProcessosRecebidos` e `tblProcessosGerados`. O resultado JSON
contém número, identificador interno, visualização e total declarado.
O valor `completa: bool` é calculado na borda a partir da cobertura
conferida, jamais armazenado no domínio.

**O GET ao vivo continua lendo somente a página atual.** No modo
offline, o IES agora aceita uma seleção explícita de páginas de uma
cassette já verificada, consolida as duas tabelas separadamente e recusa
lacunas, sobreposições, totais divergentes e processos repetidos.
Não inclui login, troca de unidade, POST ou escrita no SEI.

## Replay offline de cassette do Internet Archive

Use o [card OKF de Goiás](../cassettes/go/unidade-processos-paginado-go--ca25324d2aac.md)
e baixe o ZIP que ele referencia. Confira o SHA-1 antes de executar:

```sh
curl -fL -o /tmp/ies-go.zip \
  https://archive.org/download/todos-sei-cassettes-2026/todos-sei-cassettes-go.zip
echo '42a08cd3714bb863d136ed98e517221510a31982  /tmp/ies-go.zip' | sha1sum -c -
mkdir -p .cache/ies
cp /tmp/ies-go.zip .cache/ies/todos-sei-cassettes-go.zip
cargo run --locked -- replay unidade-processos-paginado-go --json
```

O comando público de domínio não recebe opções de cassette.
O replay usa nome de cenário e cache local.
O IES também confere internamente SHA-1 do ZIP, SHA-256
normalizado da cassette, host público permitido, schema e status 200.
Não publica cookies, HTML bruto nem URLs `infra_hash`.

Para reunir páginas já capturadas, passe vários `--passo` na ordem da
observação, por exemplo `--passo 0 --passo 2 --passo 3`. O primeiro passo
deve ser uma resposta GET da caixa; os seguintes podem ser respostas GET
ou POST de `procedimento_controlar` **já presentes no ZIP**. O comando
não reproduz o corpo do POST nem envia requisições durante o replay.
Quando uma tabela fica numa página e outra avança, a página repetida da
primeira precisa ser idêntica; revisão divergente falha fechado.
Os intervalos verificados são independentes por tabela e existem
somente enquanto a paginação está sendo consolidada. Para mesclar,
o código compara o rótulo exibido em `lnkInfraUnidade`, o modo
`hdnTipoVisualizacao` e os dois hashes de critérios
`hdnRecebidosHashCriterios` e `hdnGeradosHashCriterios`.
Falta ou divergência de contexto recusa a junção. A equivalência
não comprova identidade da sessão nem snapshot transacional. Para mesclar,
o código compara o rótulo exibido em `lnkInfraUnidade`, o modo
`hdnTipoVisualizacao` e os dois hashes de critérios
`hdnRecebidosHashCriterios` e `hdnGeradosHashCriterios`.
Falta ou divergência de contexto recusa a junção. A equivalência
não comprova identidade da sessão nem snapshot transacional. `completa: false`
significa que a cobertura não foi provada, mesmo quando a contagem
de linhas parecer compatível. `completa: true` significa somente
que os intervalos e totais declarados foram conferidos; **não prova
snapshot transacional ou imutabilidade do conjunto entre páginas**.

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
não segue redirects, não faz POST, não cria login e não repete
pedidos automaticamente. Falhas não imprimem cookies ou URL assinada.

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

O JSON contém `unidade`, `origem` e, para Recebidos e Gerados,
`processos`, `total_declarado` e `completa`. A prova de cobertura
não é exposta como estado mutável. O
[golden de saída](../tests/fixtures/caixa-v2.json)
fixa esse contrato byte a byte num teste de integração.

## Testes e limites

- UnB: 25 recebidos e 18 gerados; ambas as tabelas completas.
- PEN3: 100 recebidos de 168 e 91 gerados de 91; parcial.
- Goiás: 10 recebidos de 131 e 10 gerados de 7939; parcial.

Os três resultados foram reproduzidos offline a partir dos ZIPs
públicos, sem gravações no repositório. O `cargo test` utiliza
HTML sintético para proteger parser e transportes contra regressões.

Próximos passos: capturar o payload **completo** da navegação de página
em ambiente autorizado, reconstruir o POST a partir do formulário fresco,
verificar a resposta e restaurar filtros/página da sessão. Até isso ser
comprovado, **a paginação HTTP permanece desabilitada**.
