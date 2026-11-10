---
type: ImplementationDecision
title: Leitura dos blocos pelo menu Web observado
operation: bloco.listar
implementation: partial
status: adopted-for-scope
scope: listagem e leitura pontual de blocos pelo SEI Web sem escrita
selected_strategy: web-menu-get
alternatives: web-menu-get,rest-bloco-listar,soap-bloco
evidence_level: oraculo-todos-323eb01-e-testes-sinteticos
live_validation: not-run-unb-go
---

# Leitura dos blocos pelo menu Web observado

## Alternativas consideradas

(a) `web-menu-get`: seguir somente ações servidas na tela do Controle,
com link e origem validados, incluindo navegação da segunda página
interna por formulário servido, quando disponível. (b) `rest-bloco-listar`:
consultar um
endpoint REST não demonstrado em todas as versões. (c) `soap-bloco`:
usar SOAP apesar de a interface Web já oferecer a lista.

## Comparação

Web conserva o contexto da unidade e dos filtros da conta, mas requer
validar URLs assinadas e as tabelas. REST e SOAP não têm equivalência
demonstrada com a listagem dessa unidade e portanto não substituem
evidência de páginas reais.

## Decisão

Adotamos `web-menu-get` para a leitura da primeira página observada:
GET Controle e GET da ação de bloco realmente publicada no menu (2 GET).
Para obter o detalhe N de assinatura, seguimos apenas o link de N na
tabela servida e fazemos mais um GET (3 GET, zero POST). Para
`ies bloco listar --tipo interno --pagina 2`, a primeira página publicada
pode oferecer `frmBlocoLista`: a CLI deriva **um POST único de navegação**
com índice zero-based `1` e só aceita a segunda faixa quando contígua
com a primeira e com o mesmo total (2 GET + 1 POST). O POST altera o estado
de página da sessão, **não é operação de escrita em blocos**. A forma
do corpo original do POST não foi preservada na cassette Goiás; derivar
os pares dos controles servidos é hipótese sintética, não paridade live.
Não executamos JavaScript, não repetimos POST e não restauramos página/filtro.
Em listagem, quantidade de documentos por bloco é `null` enquanto não
observada no detalhe. Página 2 dos demais tipos e páginas acima de 2
continuam recusadas antes da rede.

## Evidências verificáveis

[Contrato do comando](../../contracts/cli/bloco-listar.md),
[matriz e oráculo](../../contracts/evidencias/bloco-listar.md) e
[teste Rust integrado](../../tests/bloco_listar.rs),
[teste vertical da CLI real com anti-IES/SQLite](../../tests/aies_bloco_cli.rs)
e [derivação da paginação](../../src/bloco_cli/paginacao.rs).
Os testes unitários de leitor derivam cenários de Todos @323eb01;
nenhum cookie, infra_hash ou URL assinada é serializado.

## Limitações e revisão

A paginação **além da segunda página interna** e a paginação do detalhe,
a leitura por número nos tipos interno/reunião, o confronto semântico
independente com os corpos originais das cassettes e a observação ao vivo
em UnB/GO ainda não passaram por validação. O teste de página 2 interna
é sintético na CLI real/anti-IES/SQLite; não comprova POST de treinamento.
Este é um recorte parcial.
Não ampliar a cobertura declarada sem testar a sessão, a contagem
exata dos pedidos e a forma real dos filtros no SEI de treinamento.
